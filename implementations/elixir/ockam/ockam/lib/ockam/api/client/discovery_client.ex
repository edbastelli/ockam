defmodule Ockam.API.Client.DiscoveryClient do
  @moduledoc """
  Client API for discovery service
  """

  alias Ockam.API.Discovery.ServiceInfo

  alias Ockam.API.Client
  alias Ockam.API.Response

  require Logger

  def register(api_route, id, self_address, metadata \\ %{}, timeout \\ 5_000) do
    service_info = %ServiceInfo{id: id, route: [], metadata: metadata}

    case Client.sync_request(
           :put,
           id,
           ServiceInfo.encode(service_info),
           api_route,
           timeout,
           self_address
         ) do
      {:ok, %Response{status: 200}} ->
        :ok

      {:ok, %Response{status: status, body: error}} ->
        {:error, {:api_error, status, error}}

      {:error, reason} ->
        {:error, reason}
    end
  end

  def list_services(access_route, discovery_route, timeout \\ 5_000, self_address \\ nil) do
    api_route = access_route ++ discovery_route

    with_self_address(self_address, fn self_address ->
      case Client.sync_request(:get, "", "", api_route, timeout, self_address) do
        {:ok, %Response{status: 200, body: service_infos}} ->
          {:ok,
           extend_routes_with_access_route(ServiceInfo.decode_list(service_infos), access_route)}

        {:ok, %Response{status: status, body: error}} ->
          {:error, {:api_error, status, error}}

        {:error, reason} ->
          {:error, reason}
      end
    end)
  end

  def extend_routes_with_access_route(service_infos, access_route) do
    Enum.map(service_infos, fn %{route: route} = service_info ->
      Map.put(service_info, :relative_route, access_route ++ route)
    end)
  end

  def with_self_address(nil, fun) do
    {:ok, call_address} = Ockam.Node.register_random_address()

    try do
      fun.(call_address)
    after
      Ockam.Node.unregister_address(call_address)
    end
  end

  def with_self_address(address, fun) do
    fun.(address)
  end
end
