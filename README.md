# shs:  A simple HTTP sink
This is a very simple HTTP/1.1 server that will print out the request and the
response (headers).  As of the first version, it will only respone at "/" and
nothing else, but will respond for GET/POST/PUT/DELETE requests.

## Usage
This is just a matter of running `shs` and you will have a webserver bound to
`localhost:8000` that will respond with a 200 and a body of "ok" to any
GET/POST/PUT/DELETE request that you send at it.

Run with defaults:
```
shs
```

Set a custom response:
```
shs -r "Monkeys are great!"
```

Bind to a different address/port:
```
shs -b "192.168.0.100" -p 8080
```

## TODO
* Allow more options on the command line to be more dynamic in what the
  response body contents are (just "ok" for now).