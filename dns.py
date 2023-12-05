"""Redirect HTTP requests to another server."""
from mitmproxy import http
from mitmproxy.connection import Server
from mitmproxy.net.server_spec import ServerSpec
import logging

from mitmproxy.addonmanager import Loader
from mitmproxy.log import ALERT
logger = logging.getLogger(__name__)

def proxy_address(flow: http.HTTPFlow) -> tuple[str, int]:
    # Poor man's loadbalancing: route every second domain through the alternative proxy.
    if flow.request.pretty_host == "lake.nearhat":
        return ("localhost", 55313)
    elif flow.request.pretty_host == "rpc.nearhat":
        return ("localhost", 55364)
    else:
        return ("localhost", 3000)

def request(flow: http.HTTPFlow) -> None:
    address = proxy_address(flow)
    flow.request.host = address[0]
    flow.request.port = address[1]