#!/usr/bin/env python3

"""Really simple script to push a script to AntiRaid"""
from dotenv import load_dotenv

load_dotenv()  # take environment variables

import os
import pathlib
import requests

API_URL = "https://splashtail-staging.antiraid.xyz/"
NEEDED_CAPS = ["discord:get_audit_logs", "discord:create_message"] # List of needed capabilities
EVENTS = ["MESSAGE", "INTERACTION_CREATE"] # List of events to listen to
USE_BUNDLED_TEMPLATING_TYPES = True # Use bundled types
TEMPLATE_NAME = "templating-template" # Name of the template
IGNORE_FILES = [
    ".env",
    ".env.sample",
    ".git",
    ".gitignore",
    ".gitmodules",
    "README.md",
    "push.py",
    "requirements.txt",
    "LICENSE",
    ".darklua.json5"
]
SHOP_DESCRIPTION = "Automatically slow down chat based on configuration. See https://github.com/anti-raid/auto-slowdown for configuration options"
SHOP_FRIENDLY_VERSION = "Auto Slowdown"
SHOP_TEMPLATE_V = "0.3.0"

contents = {}
for path in pathlib.Path(".").rglob("*"):
    ignored = False
    for f in IGNORE_FILES:
        if str(path).startswith(f):
            ignored = True
            break
    
    if ignored:
        continue

    if USE_BUNDLED_TEMPLATING_TYPES and str(path).startswith("templating-types"):
        continue # use bundled types

    print(path)

    if path.is_file():
        with open(path, "r") as f:
            contents[str(path)] = f.read()

if USE_BUNDLED_TEMPLATING_TYPES:
    NEEDED_CAPS.append("assetmanager:use_bundled_templating_types")

api_token = os.getenv("API_TOKEN")
if not api_token:
    raise ValueError("API_TOKEN is not set in the environment variables")

guild_id = os.getenv("GUILD_ID")
if not guild_id:
    raise ValueError("GUILD_ID is not set in the environment variables")

error_channel = os.getenv("ERROR_CHANNEL")
if not error_channel:
    raise ValueError("ERROR_CHANNEL is not set in the environment variables")

res = requests.post(f"{API_URL}/guilds/{guild_id}/settings", 
    json={
        "fields": {
            "name": TEMPLATE_NAME,
            "friendly_name": SHOP_FRIENDLY_VERSION,
            "description": SHOP_DESCRIPTION,
            "version": SHOP_TEMPLATE_V,
            "language": "luau",
            "paused": False,
            "allowed_caps": NEEDED_CAPS,
            "events": EVENTS,
            "content": contents,
            "error_channel": error_channel,
        },
        "operation": "Create",
        "setting": "script_shop"
    },
    headers={
        "Authorization": api_token,
        "Content-Type": "application/json"
    }
)

message: str = None
# Handle template already exists
if res.status_code == 400:
    message = res.json().get("message")

if res.ok:
    print("Script pushed successfully")
else:
    print(f"Failed to push script with status code {res.status_code}")
    print("Response:")
    print(res.json())
    exit(1)