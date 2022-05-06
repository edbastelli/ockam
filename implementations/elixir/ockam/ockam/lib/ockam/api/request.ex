defmodule Ockam.API.Request do
  @moduledoc """
  Okcam request-response API request
  """

  defstruct [:id, :path, :method, :body, :from_route, :to_route]

  @max_id 65_534

  @method_schema {:enum, [:get, :post, :put, :delete, :patch]}
  @schema {:map, [:id, :path, {:method, @method_schema}, :has_body]}

  def encode(request) when is_map(request) do
    body =
      case request.body do
        binary when is_binary(binary) -> binary
        ## TODO fail instead?
        other -> CBOR.encode(other)
      end

    request =
      case byte_size(body) > 0 do
        true -> Map.put(request, :has_body, true)
        false -> Map.put(request, :has_body, false)
      end

    base = MiniCBOR.encode(request, @schema)
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

  def from_message(%Ockam.Message{
        payload: payload,
        onward_route: onward_route,
        return_route: return_route
      }) do
    with {:ok, %__MODULE__{} = request} <- decode(payload) do
      {:ok, %{request | from_route: return_route, to_route: onward_route}}
    end
  end

  def to_message(%__MODULE__{to_route: to_route} = request, return_route) do
    %Ockam.Message{
      payload: encode(request),
      onward_route: to_route,
      return_route: return_route
    }
  end
end
