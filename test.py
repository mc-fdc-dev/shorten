import requests


r = requests.post("https://shor.f5.si/shorten", json={
    "url": "http://google.com/"
})

print(r.text)