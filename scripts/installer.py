import urllib.request
import json
import os
import shutil
from zipfile import ZipFile
from io import BytesIO

HOME = os.environ["HOME"]
SCOUT_DIR = os.path.join(HOME, "scout-lang")

if not os.path.exists(SCOUT_DIR):
    os.makedirs(SCOUT_DIR)
else:
    shutil.rmtree(os.path.join(SCOUT_DIR, "scout-std"))


url = "https://api.github.com/repos/maxmindlin/scout-lang/releases/latest"
r = urllib.request.urlopen(url)
data = json.loads(r.read().decode(r.info().get_param("charset") or "utf-8"))
version = data["name"]

zip_url = f"https://github.com/maxmindlin/scout-lang/archive/refs/tags/{version}.zip"
zip_resp = urllib.request.urlopen(zip_url)
myzip = ZipFile(BytesIO(zip_resp.read()))

std_to_replace = f"scout-lang-{version[1::]}/"
std_file_start = std_to_replace + "scout-std/"
for file in myzip.infolist():
    if std_file_start in file.filename:
        replaced = file.filename.replace(std_to_replace, "")
        if replaced == std_to_replace:
            continue
        file.filename = replaced
        myzip.extract(file, path=SCOUT_DIR)

installer = f"curl --proto '=https' --tlsv1.2 -LsSf https://github.com/maxmindlin/scout-lang/releases/download/{version}/scout-installer.sh | sh"

os.system(installer)
