"""Redirect HTTP requests to another server."""
from mitmproxy import http
import logging
import os

from mitmproxy.addonmanager import Loader
from mitmproxy.log import ALERT
logger = logging.getLogger(__name__)

def proxy_address(flow: http.HTTPFlow) -> tuple[str, int]:
    if flow.request.pretty_host == "lake.nearhat":
        return ("localhost", int(os.getenv('NEARHAT_LAKE_S3_PORT')))
    elif flow.request.pretty_host == "rpc.nearhat":
        return ("localhost", int(os.getenv('NEARHAT_RPC_PORT')))
    else:
        return ("localhost", 3000)

def request(flow: http.HTTPFlow) -> None:
    address = proxy_address(flow)
    flow.request.host = address[0]
    flow.request.port = address[1]