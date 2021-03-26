# shs:  A simple HTTP sink
This is a very simple HTTP/1.1 server that will print out the request and the
response (headers).  As of the first version, it will only respone at "/" and
nothing else, but will respond for GET/POST/PUT/DELETE requests.

## TODO
* Create a generic handler to respond to any path.
* Allow more options on the command line to be more dynamic in what the
  response body contents are (just "ok" for now).