import requests


r = requests.get("https://sh.tuna2134.jp", headers={
    "Origin": "https://sh.tuna2134.jp",
    "Access-Control-Request-Method": "GET"
})

print(r.text)
print(r.headers)