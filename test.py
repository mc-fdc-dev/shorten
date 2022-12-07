import requests


r = requests.post("http://localhost:8001/shorten", json={
    "url": "https://www.megasoft.co.jp/mifes/seiki/s310"
})

print(r.text)