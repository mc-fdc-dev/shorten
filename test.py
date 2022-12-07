import requests


r = requests.get("https://sh.tuna2134.jp/test", headers={
    "Origin": "https://review2.tuna2134.jp/test",
    "Access-Control-Request-Method": "GET"
})

print(r.text)
print(r.headers)