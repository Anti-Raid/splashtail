import os
from bs4 import BeautifulSoup

with open("data/events/events_tab.html") as f:
    file_tab = f.read()

parsed_html = BeautifulSoup(file_tab, features="html.parser")

# Find the tbody
tbody = parsed_html.find("tbody")
if not tbody:
    raise ValueError("No tbody found in the HTML file.")

events = []
# Find every tr in the tbody
for tr in tbody.find_all("tr"):
    # Find the first td in the tr
    tds = tr.find_all("td")
    
    name = tds[0]
    if not name:
        raise ValueError("No name found in the td.")
    description = tds[1]
    if not description:
        raise ValueError("No description found in the td.")

    # Find the first a in the name td
    a = name.find("a")
    if not a:
        raise ValueError("No link found in the name td.")

    # Link text
    link_text = a.get_text(strip=True)
    if not link_text:
        raise ValueError("No link text found in the a tag.")

    events.append({"name": link_text, "description": description.get_text(strip=True)})

print(events)

# Convert to rust definition
rust_def_str = "[\n"
num_events = 0
for event in events:
    if event["name"] in ["Hello", "Ready", "Resumed", "Reconnect", "Invalid Session", "Guild Create", "Guild Delete"]:
        continue
    
    if event["name"] == "Message Create":
        event["name"] = "Message"
    
    name = event["name"].replace(" ", "_").upper()

    rust_def_str += f"  \"{name}\", // {event['description']}\n"
    num_events += 1

rust_def_str += "];\n"

rust_arraydef = f"pub const EVENT_LIST: [&str; {num_events}] = {rust_def_str}"
print(rust_arraydef)