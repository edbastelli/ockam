defmodule Ockam.Services.API.Discovery do
  @moduledoc """
  API for discovery service

  Methods:

  :get, path: "" - list all services
  :get, path: service_id - get one service info
  :put, path: service_id, body: `Ockam.API.Discovery.ServiceInfo` - register a service
  """

  use Ockam.Services.API

  alias Ockam.API.Discovery.ServiceInfo

  @impl true
  def setup(options, state) do
    storage = Keyword.get(options, :storage, Ockam.Services.Discovery.Storage.Memory)
    storage_options = Keyword.get(options, :storage_options, [])

    {:ok, Map.put(state, :storage, {storage, storage.init(storage_options)})}
  end

  @impl true
  def handle_request(%Request{method: :get, path: ""}, state) do
    with {infos, state} <- list(state) do
      {:reply, :ok, ServiceInfo.encode_list(infos), state}
    end
  end

  def handle_request(%Request{method: :get, path: id}, state) do
    with {{:ok, info}, state} <- get(id, state) do
      {:reply, :ok, ServiceInfo.encode(info), state}
    end
  end

  def handle_request(%Request{method: :put, path: id, body: body}, state) do
    with {:ok, %{route: route, metadata: metadata}} <- ServiceInfo.decode(body),
         :ok <- register(id, route, metadata, state) do
      {:reply, :ok, "", state}
    end
  end

  def handle_request(%Request{method: _other}, _state) do
    {:error, :method_not_allowed}
  end

  def list(state) do
    with_storage(state, fn storage_mod, storage_state ->
      storage_mod.list(storage_state)
    end)
  end

  def get(id, state) do
    with_storage(state, fn storage_mod, storage_state ->
      storage_mod.get(id, storage_state)
    end)
  end

  def register(id, route, metadata, state) do
    with_storage(state, fn storage_mod, storage_state ->
      storage_mod.register(id, route, metadata, storage_state)
    end)
  end

  def with_storage(state, fun) do
    {storage_mod, storage_state} = Map.get(state, :storage)
    {result, new_storage_state} = fun.(storage_mod, storage_state)
    {result, Map.put(state, :storage, {storage_mod, new_storage_state})}
  end
end
