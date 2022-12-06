import requests


r = requests.post("http://localhost:8001/shorten", json={
    "url": "https://google.co.jp"
})

print(r.text)