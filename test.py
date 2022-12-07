import requests


r = requests.post("http://localhost:8001/shorten", json={
    "url": "https://tuna2134.jp"
})

print(r.text)