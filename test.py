import requests


r = requests.post("http://localhost:8000/shorten", json={
    "url": "https://google.com"
})

print(r.text)