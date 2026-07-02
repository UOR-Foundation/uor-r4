# Nginx Map for Domain & Port

- **Conversation ID:** `8a63d8d8-8f02-4d8c-b447-1daa163a6fc3`
- **Date:** 2023-10-31

---

### User (2023-10-31 14:36:35)

write me an nginx map function to allow a domain and optionally a port

### Assistant (2023-10-31 14:36:44)

You can use the Nginx `map` module to conditionally allow a domain and optionally a port in your Nginx configuration. Here's an example of how you can achieve this:

```nginx
http {
    map $host $allowed_domain {
        default 0;  # By default, deny all requests
        example.com 1;  # Allow requests with the domain "example.com"
        sub.example.com 1;  # Allow requests with the subdomain "sub.example.com"
    }

    server {
        listen 80;
        server_name _;

        if ($allowed_domain = 0) {
            return 403;  # Deny access for any other domains
        }

        location / {
            # Your other Nginx configuration directives
        }
    }
}
```

In this example, we use the `map` directive to create a variable called `$allowed_domain`, which maps the value of the `$host` variable (the incoming domain) to 1 if it's allowed and 0 if it's not. You can add more domain entries as needed.

Inside the `server` block, we check the value of `$allowed_domain`. If it's 0 (i.e., the domain is not in the map), we return a 403 Forbidden status, denying access to the server for any other domain. If it's 1 (i.e., the domain is in the map), the request is allowed to proceed.

This configuration snippet allows you to control access based on the domain. If you also want to optionally check for a specific port, you would need to use an `if` block with a more complex condition. Here's an example of how you can do that:

```nginx
http {
    map $host:$server_port $allowed_domain_port {
        default 0;
        example.com:80 1;
        sub.example.com:8080 1;
    }

    server {
        listen 80;
        server_name _;

        if ($allowed_domain_port = 0) {
            return 403;
        }

        location / {
            # Your other Nginx configuration directives
        }
    }
}
```

In this updated configuration, we check both the domain and the port. The `map` directive now maps the combination of `$host` and `$server_port`. Adjust the entries in the map to match the allowed domain and port combinations you want to permit.

### User (2023-10-31 14:38:26)

write me a regex expression to for an nginx map function that allows a domain and optionally a port

### Assistant (2023-10-31 14:38:34)

You can use a regular expression to match a domain and optionally a port in an Nginx `map` block. Here's an example of a regular expression that allows a domain and an optional port:

```nginx
http {
    map $host$request_uri $allowed_domain {
        ~^(?<domain>[a-z0-9.-]+)(:(?<port>\d+))?(/.*)?$  $domain$port;
        default 0;  # Deny by default
        example.com 80;  # Allow requests to "example.com" on port 80
        sub.example.com 8080;  # Allow requests to "sub.example.com" on port 8080
    }

    server {
        listen 80;
        server_name _;

        if ($allowed_domain = 0) {
            return 403;  # Deny access for any other domains or ports
        }

        location / {
            # Your other Nginx configuration directives
        }
    }
}
```

In this configuration:

- We use a regular expression in the `map` block that captures the domain and an optional port. The regular expression `~^(?<domain>[a-z0-9.-]+)(:(?<port>\d+))?(/.*)?$` captures the domain in the `domain` named capture group and the port in the `port` named capture group. The `(/.*)?$` part allows for any additional URI path if needed.

- The `map` block then maps the extracted domain and port to a value. If a match is found, it returns the concatenated `domain` and `port` values, allowing the domain and an optional port. If there's no match, it defaults to 0.

- Inside the `server` block, we check the value of `$allowed_domain`. If it's 0 (i.e., the domain or port doesn't match any allowed values), we return a 403 Forbidden status, denying access to the server for any other combinations.

You can modify the regular expression to match the specific domain and port combinations you want to allow.

