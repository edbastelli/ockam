defmodule Ockam.API.Response do
  @moduledoc """
  Okcam request-response API response
  """

  alias Ockam.API.Request

  defstruct [:id, :request_id, :status, body: "", from_route: [], to_route: []]

  @max_id 65_534

  @schema {:map, [:id, :request_id, :status, :has_body]}

  def encode(response) when is_map(response) do
    body =
      case response.body do
        binary when is_binary(binary) -> binary
        ## TODO fail instead?
        other -> CBOR.encode(other)
      end

    response =
      case byte_size(body) > 0 do
        true -> Map.put(response, :has_body, true)
        false -> Map.put(response, :has_body, false)
      end

    base = MiniCBOR.encode(response, @schema)
    base <> body
  end

  def decode(data) when is_binary(data) do
    case MiniCBOR.decode(data, @schema) do
      {:ok, decoded, body} ->
        has_body = Map.get(decoded, :has_body)
        body_present = byte_size(body) > 0

        case {has_body, body_present} do
          {same, same} ->
            {:ok, struct(__MODULE__, Map.put(decoded, :body, body))}

          {true, false} ->
            {:error, {:decode_error, :missing_body, data}}

          {false, true} ->
            {:error, {:decode_error, :unexpected_body, data}}
        end

      other ->
        {:error, {:decode_error, other, data}}
    end
  end

  def gen_id() do
    :rand.uniform(@max_id)
  end

  def reply_to(%Request{id: request_id, from_route: route}, status, body) do
    %__MODULE__{
      id: gen_id(),
      request_id: request_id,
      status: status,
      body: body,
      to_route: route
    }
  end

  def from_message(%Ockam.Message{
        payload: payload,
        onward_route: onward_route,
        return_route: return_route
      }) do
    with {:ok, %__MODULE__{} = request} <- decode(payload) do
      {:ok, %{request | from_route: return_route, to_route: onward_route}}
    end
  end

  def to_message(%__MODULE__{to_route: to_route} = response, return_route) do
    %Ockam.Message{
      payload: encode(response),
      onward_route: to_route,
      return_route: return_route
    }
  end
end
