# CORS Setup for Multi-Domain

- **Conversation ID:** `67cb49b6-0ed0-8001-90b3-2707f7c83159`
- **Date:** 2025-03-07

---

### User (2025-03-07 14:32:05)

Can you tell me how to correctly use CORS in the contest of a multi-domain (FQDN) environment that passes URL's through multiple nginx proxies configured to block CORS that is not configured improperly - and an application that requires web authentication for REDIS, POSTGRES through front end api http interfaces that accept typical HTTP options

### Assistant (2025-03-07 14:32:21)

To provide precise guidance on configuring CORS in your multi-domain environment, could you clarify the following details:

1. **Nginx Proxy Setup**:
   - Are the multiple Nginx proxies operating as load balancers, reverse proxies, or both?
   - Are they all under your control, or are some third-party proxies involved?

2. **CORS Blocking**:
   - Is the intent to selectively allow CORS for specific domains, or completely block it unless explicitly configured?
   - Are you looking for best practices in setting `Access-Control-Allow-Origin`, `Access-Control-Allow-Methods`, and `Access-Control-Allow-Headers`?

3. **Authentication Setup**:
   - How is authentication handled for Redis and Postgres (e.g., API tokens, OAuth, JWT)?
   - Do requests need to pass credentials (`Access-Control-Allow-Credentials: true`)?

4. **Frontend API Usage**:
   - What is the frontend framework or technology (e.g., React, Angular, Vue)?
   - Is it hosted on a separate domain/subdomain from the backend API?

5. **HTTP Methods & Security**:
   - Do you require support for all HTTP methods (`OPTIONS`, `GET`, `POST`, etc.), or only a subset?
   - Do you want any additional security constraints (e.g., rate limiting, CSRF protection)?

Let me know so I can tailor the response accordingly!

### User (2025-03-07 14:35:50)

1. There are nginx proxies in front of all of the different URL's being accessed. We control these proxies. They are acting as reverse proxies and WAF's.
2. We do not want to Allow domains that are not part of the application - so yes, best practices to configure all proxy front ends
3. The client api and worker api use API tokens
4. The frontend and backend framework are both python (Django), and the domains are different.
5. Yes all methods. Tell me how to properly use CSRF because I am receiving those errors currently.

### Assistant (2025-03-07 14:36:03)

Thanks for the details! I'll gather best practices for configuring CORS in your multi-domain Django-Nginx environment, ensuring only allowed domains access your APIs while preventing CORS errors. Additionally, I'll provide guidance on properly handling CSRF issues in Django, particularly in an API-based authentication setup using API tokens.

I'll update you once I have the findings ready.

### Assistant (2025-03-07 14:35:51)

{"task_violates_safety_guidelines": false, "user_def_doesnt_want_research": false, "response": "Thanks for the details! I'll gather best practices for configuring CORS in your multi-domain Django-Nginx environment, ensuring only allowed domains access your APIs while preventing CORS errors. Additionally, I'll provide guidance on properly handling CSRF issues in Django, particularly in an API-based authentication setup using API tokens.\n\nI'll update you once I have the findings ready.", "title": "CORS and CSRF Configuration for Multi-Domain Django-Nginx", "prompt": "Research best practices for configuring CORS in a multi-domain Django-Nginx environment where multiple Nginx proxies act as reverse proxies and WAFs. Ensure that only allowed domains (part of the application) can access the APIs. The APIs use API token authentication, and both frontend and backend are built with Django but are hosted on different domains.\n\nProvide a detailed guide on:\n1. Correctly configuring CORS headers in Nginx to prevent unauthorized cross-origin requests while allowing legitimate application domains.\n2. Setting up Django's CORS and CSRF protection correctly, ensuring that CSRF errors do not occur while maintaining security.\n3. Configuring Django to properly handle API token authentication in a CSRF-protected manner.\n4. Ensuring that HTTP methods (OPTIONS, GET, POST, etc.) are correctly handled across Nginx and Django settings.\n5. Addressing potential issues that might arise due to WAF rules conflicting with CORS or CSRF settings and how to mitigate them.\n\nFormat the response as a structured guide with step-by-step configurations for both Nginx and Django settings."}

### Assistant (2025-03-07 14:44:29)

# CORS and CSRF Configuration Guide for a Multi-Domain Django + Nginx Setup

In a multi-domain environment with Django backends and Nginx proxies (including WAF functionality), it's critical to configure Cross-Origin Resource Sharing (CORS) and Cross-Site Request Forgery (CSRF) protection properly. The goal is to **allow only trusted domains** (e.g. your front-end domain) to access the Django API domain, while blocking unauthorized origins. We also need to accommodate API token authentication and ensure that CSRF protection is correctly handled (or safely disabled for token-based auth) to avoid unwanted 403 errors. Below is a step-by-step guide covering Nginx configuration, Django settings for CORS/CSRF, token authentication, HTTP method handling, and WAF considerations.

## 1. Configuring CORS Headers in Nginx (Allow Only Whitelisted Domains)

**Goal:** Configure Nginx to send CORS headers that **only allow your known front-end domains** to call the API, and block all other origins by not sending any CORS header. This protects the API from unauthorized cross-origin requests. Nginx will inspect the request’s `Origin` header and respond with the appropriate CORS headers if the origin is allowed.

**Key Points & Steps:**

- **Whitelist the allowed origins:** Identify all domains that need access (e.g. your front-end application domains). Do **not** use `*` wildcard for `Access-Control-Allow-Origin` in production, and do not attempt to list multiple domains in one header (browsers don’t support multiple origins in a single header)【2†L196-L203】. Instead, you'll configure Nginx to dynamically echo back the single allowed origin if it matches your whitelist.

- **Use Nginx `map` for allowed origins:** In your Nginx config (e.g. a http or server block), define a mapping of `$http_origin` to an `$allowed_origin` variable. This map will set `$allowed_origin` to the incoming Origin **only if** it matches one of the trusted domains, or empty otherwise. For example: 

  ```nginx
  map "$http_origin" $allowed_origin {
      default         "";
      "~^https?://(frontend1.example.com|frontend2.example.com)$"  "$http_origin";
  }
  ```
  This regex allows `https://frontend1.example.com` or `https://frontend2.example.com` (adjust for your domains and include ports if needed). If the Origin matches, `$allowed_origin` is set to that origin; if not, it remains empty. This is a safer approach than using multiple `if` statements and avoids issues with Nginx’s `if` pitfalls【3†L290-L298】【19†L282-L291】.

- **Add CORS headers for allowed origins:** Inside the Nginx `server` or `location` block that proxies your API, emit CORS headers **only if** `$allowed_origin` is not empty. For example:

  ```nginx
  server {
      # ... (other config like proxy_pass, etc.)
      location /api/ {
          # Include the CORS mapping logic
          # (or 'include cors_configs.conf' if you put the map in a snippet)
  
          # Add CORS response headers when origin is allowed
          if ($allowed_origin != "") {
              add_header Access-Control-Allow-Origin "$allowed_origin" always;
              add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS" always;
              add_header Access-Control-Allow-Headers "Content-Type, Authorization, X-CSRFToken, X-Requested-With" always;
              add_header Access-Control-Allow-Credentials "true" always;
          }
          # Handle preflight OPTIONS requests
          if ($request_method = OPTIONS) {
              add_header Access-Control-Max-Age 1728000;  # cache preflight response (20 days here)
              add_header Content-Type "text/plain; charset=UTF-8";
              add_header Content-Length 0;
              return 204;
          }
          # ... proxy_pass to Django upstream, etc.
      }
  }
  ```
  **Explanation:** The `if ($allowed_origin != "")` block ensures that the CORS headers are only added for requests from valid origins【19†L290-L299】. We specify:
  - `Access-Control-Allow-Origin` as the `$allowed_origin` (the request’s origin if it was in our whitelist).
  - `Access-Control-Allow-Methods` to include all HTTP methods your API needs (GET, POST, PUT, DELETE, OPTIONS, etc.)【19†L295-L302】.
  - `Access-Control-Allow-Headers` to include necessary request headers. At minimum, include `Content-Type` (for JSON POSTs) and `Authorization` (since you use token auth). We also list `X-CSRFToken` and `X-Requested-With` as common headers for AJAX/CSRF, and any other custom headers your front-end might send. Make sure **not** to omit `Authorization`, or the browser’s preflight will fail when it’s needed.
  - `Access-Control-Allow-Credentials` to `true` **only if** you need cookies/auth credentials. For token authentication you might not strictly need it (since tokens are sent in headers, not cookies), but if your front-end also needs to send cookies (e.g. a CSRF cookie or session cookie) or if you use `withCredentials`, then set this to `true`. **Note:** When using `Allow-Credentials: true`, you must specify a single allowed origin (no `*`), which we’re doing by whitelisting. This ensures secure cross-site cookies if ever used【19†L292-L299】.
  - We use the `always` flag on `add_header` to include these headers even on error responses or redirects【3†L235-L243】. This is important so that if Django returns a 403 CSRF error or the WAF blocks something, the error response still includes CORS headers and can be read by the front-end JS (otherwise you get a generic CORS failure with no details).

- **Handle the preflight (OPTIONS) request:** The second `if` catches HTTP OPTIONS requests. It returns **HTTP 204 No Content** with the relevant headers【19†L303-L310】. This means Nginx itself will respond to preflight checks without forwarding to Django, which is efficient and avoids unnecessary load. We also send `Access-Control-Max-Age` to cache the preflight response for some time (here 1728000 seconds ~ 20 days) so the browser doesn’t repeat preflights too often. Adjust or omit the max-age as appropriate (e.g. shorter during development).

- **Result:** With the above Nginx configuration, any request coming from an origin not in your list will **not** receive an `Access-Control-Allow-Origin` header in the response, causing the browser to block it. Allowed domains will get the appropriate headers and be able to interact with the API. This dynamic approach ensures only one origin is reflected back (avoiding the “multiple values” CORS error)【3†L270-L278】. As one source explains, making the header dynamic per request lets you securely allow multiple specific origins without using the insecure `*` wildcard【19†L311-L319】【2†L196-L204】.

- **Placement:** Include this configuration in the Nginx instance that clients connect to. In a chain of proxies, that’s likely the outermost Nginx (the one acting as WAF/reverse proxy to the world). If you have multiple layers of Nginx, you can implement CORS at the edge or at the app’s Nginx – just ensure only one adds the headers to avoid duplication. It’s usually simplest at the edge layer so that the headers are added *before* any potential WAF filtering and the client sees them directly.

- **Verify Nginx behavior:** After configuring, test from an allowed origin (you should see `Access-Control-Allow-Origin: <your-domain>` in the response) and from a disallowed origin (the response should have **no** ACAO header, resulting in a CORS error in browser). This confirms Nginx is correctly gating access.

## 2. Django Settings for CORS and CSRF

On the Django side, you need to configure **which origins are allowed** to make requests, and properly set up CSRF trusted domains. Django doesn’t send CORS headers by default, but you can use the **`django-cors-headers`** module to help, and you must adjust CSRF settings for cross-domain scenarios.

**CORS Setup in Django:**

- **Install and enable django-cors-headers:** This widely used package can automatically add CORS headers to Django responses. (The Django Rest Framework team also recommends using middleware to handle CORS【9†L143-L151】.) Install via pip and add `'corsheaders'` to `INSTALLED_APPS`【22†L189-L197】. Then add the middleware `'corsheaders.middleware.CorsMiddleware'` high in the `MIDDLEWARE` list (above Django’s CommonMiddleware and AuthenticationMiddleware)【22†L201-L209】. Placing it early ensures it can add headers even on responses like redirects or errors before they are sent.

- **Allow only specific origins:** In Django settings.py, configure the whitelist of origins that can make requests:
  
  ```python
  CORS_ALLOWED_ORIGINS = [
      "https://frontend1.example.com",
      "https://frontend2.example.com",
  ]
  # or use regex patterns:
  # CORS_ALLOWED_ORIGIN_REGEXES = [r"^https://.*\.example\.com$"]  # e.g. allow any subdomain of example.com
  ```
  This ensures Django will only send `Access-Control-Allow-Origin` for these domains【22†L231-L240】. If a request comes from elsewhere, it will be denied (no CORS headers). **Do not** set `CORS_ALLOW_ALL_ORIGINS = True` in production, as that would allow any site to access your API【22†L255-L260】. Keep the list tight to your known domains.

- **Allow credentials if needed:** If your front-end needs to include cookies or HTTP auth, set `CORS_ALLOW_CREDENTIALS = True`. This will allow responses to include `Access-Control-Allow-Credentials: true`. (Remember, with credentials, wildcard origins are not allowed.) If you are purely using token auth with no cookies, you can leave this False (the default) – no credentials means no cookies are exchanged.

- **Expose any custom headers:** By default, browsers only let the front-end code access certain response headers. If you plan to send important data in a custom header (e.g. if you decide to send a CSRF token or user info in a header), list it in `CORS_EXPOSE_HEADERS`. For example: `CORS_EXPOSE_HEADERS = ["X-CSRFToken"]` if you send CSRF token in a header so JS can read it【8†L179-L188】. This setting isn’t strictly required for basic functionality, but it’s useful if your JS needs to read custom response headers.

- **Django’s CORS vs Nginx’s CORS:** You have a choice: handle CORS in Nginx (as in section 1) **or** in Django (via this middleware), or even both for robustness. Many prefer letting Nginx handle it for performance and simplicity. If you use django-cors-headers, it will add the same headers to responses from Django. Ensure you don’t double-add conflicting headers. In practice, if Nginx already added `Access-Control-Allow-Origin`, Django adding it again with the same value is usually okay (it would just overwrite or be filtered out to one value). To avoid confusion, you might disable Django’s CORS in production if Nginx is doing it. However, using django-cors-headers is very handy for local development (where you might not have Nginx) and ensures alignment with Django’s CSRF handling. It’s known to work correctly with DRF APIs out-of-the-box【9†L143-L151】. A safe approach: use Django’s CORS in development, but in production rely on Nginx’s stricter control.

**CSRF Protection in Django (Cross-Domain Considerations):**

Django’s CSRF protection by default only trusts same-origin requests. In a multi-domain setup, you need to adjust settings so that your front-end domain’s requests are recognized as safe. Key configurations:

- **CSRF_TRUSTED_ORIGINS:** Add your front-end domain(s) here. This tells Django to **allow cross-site POST/PUT/DELETE requests from those origins**, bypassing the strict referer/origin check【11†L179-L187】. For example:
  ```python
  CSRF_TRUSTED_ORIGINS = ["https://frontend1.example.com", "https://frontend2.example.com"]
  ```
  Without this, Django would reject a POST from `frontend1.example.com` to `api.example.com` with a 403 error: “CSRF Failed: Referer or Origin checking failed”. By trusting the origin, you indicate these are legitimate hosts that may make POSTs.

- **Ensuring the CSRF token is included:** If you use Django’s session authentication or any cookie-based interaction on the API side, the front-end must send the CSRF token with unsafe requests. Normally, Django sets a CSRF token cookie (`csrftoken`) on GET requests, and your JS should read that cookie and send it in an `X-CSRFToken` header for subsequent POST/PUT… (or embed it in forms). **However, in cross-domain setups, the front-end JS cannot read cookies set on another domain due to browser security**【6†L114-L123】【8†L167-L175】. This means if your API only sets a CSRF cookie on its own domain, your JS on the other domain can’t directly access it. 

  **Options to handle CSRF tokens cross-domain:**
  - *Preferred (for APIs with token auth):* Use token authentication and **skip using Django’s CSRF tokens** entirely for those API calls (more on this in the next section). This is simpler because it avoids cookies.
  - *If you must use Django’s CSRF token:* You have to **deliver the CSRF token to the client** in a way it can read. One method is to have an API endpoint that returns the CSRF token in the response body or a header, which the JS can fetch *after* logging in. For example, you could create an endpoint `/api/get-csrf/` that when called (with credentials) responds with the CSRF token in a JSON or in a custom header, and expose that header via `CORS_EXPOSE_HEADERS`【8†L179-L187】. The front-end could call this endpoint on initial load to obtain the token, then include it on later requests. Another approach is to set the CSRF cookie with `SameSite=None` and `Secure` so that it *will* be included cross-site (in a `withCredentials` XHR), but even then your JS can’t read it directly; you’d still need to grab it from a response. The django-cors-headers documentation suggests using a custom middleware to expose the CSRF cookie value as a header for such scenarios【8†L179-L187】【8†L184-L192】.
  
  - If you do go this route, ensure `Access-Control-Allow-Credentials: true` and that your XHR/fetch is sending credentials, so the CSRF cookie is included in requests and can be set by the API domain. Also, confirm that `CSRF_COOKIE_SAMESITE = 'None'` and `CSRF_COOKIE_SECURE = True` in Django settings so the cookie is allowed cross-site. This setup gets complex, which is why many APIs opt for token auth and CSRF exemption instead.

- **Avoiding CSRF errors in practice:** If you find that despite trusting the origin, you still get “CSRF token missing or incorrect” errors, it usually means the token wasn’t provided or didn’t match. Double-check that:
  - The front-end is sending the `X-CSRFToken` header (or CSRF token in form data) on unsafe requests.
  - The CSRF cookie from the API domain is being set in the browser (look for `csrftoken` cookie). If not, you may need to hit a GET view on the API domain that sets the cookie (Django only sets it on a view rendered with a csrf_token or when CSRF middleware processes a safe request).
  - Your `CSRF_TRUSTED_ORIGINS` is exactly matching the scheme+domain of the requests (include `https://` prefix as required since Django 4.0+).
  
  If everything is correct and it’s still failing, consider the approach in the next section (disabling CSRF for token-auth APIs), because maintaining CSRF across domains can be tricky.

## 3. API Token Authentication and CSRF Handling in Django

Your APIs use token authentication, meaning each request includes an API token (e.g. in the `Authorization` header) instead of relying on session cookies. This has implications for CSRF:

**Understanding CSRF with Token Auth:**

- CSRF attacks primarily target **cookie-based** authentication (sessions). If an attacker can trick a user’s browser to send a **cookie-backed** request (e.g. submit a malicious form to your API domain where the user is logged in), it could perform unwanted actions. CSRF tokens solve that by requiring a secret token that only scripts from your site know.

- With token auth, the browser will not automatically attach the token to requests – your JS code manually adds the token to the header. A malicious site cannot read or steal that token from the victim’s browser unless your JS or storage is compromised, and cannot force the browser to include an Authorization header on its own. Also, because of CORS, a rogue site can’t even make a valid AJAX call to your API domain and receive data *unless* your API explicitly allows that origin (which it doesn’t). These factors mean that **CSRF protection is not strictly necessary for pure token-based authentication** in the same way it is for cookies. In fact, Django’s SessionAuthentication is the one that requires CSRF tokens, whereas an API using only token auth can safely be exempted【34†L39-L46】. 

- **Bottom line:** If your Django views strictly require a correct API token for any state-changing request, an attacker cannot forge such a request unless they have the token. Therefore, it’s common to **disable CSRF checks on API endpoints that use token auth**, to avoid needless CSRF errors.

**Configuring Django for Token Auth:**

- **Use Django REST Framework (DRF) TokenAuth or similar** – If you haven’t already, consider using DRF’s token authentication or JSON Web Tokens (JWT) for your API. DRF provides a TokenAuth system where each user has a token string. The client sends `Authorization: Token <key>` on requests. DRF will authenticate `request.user` based on that. Another popular option is JWT (e.g. via djangorestframework-simplejwt) with `Authorization: Bearer <jwt>`. The key is that authentication is done via header, not session.

- **Disable SessionAuthentication for APIs:** In DRF’s settings, if you have `DEFAULT_AUTHENTICATION_CLASSES`, include the token auth class and remove session authentication. For example:
  ```python
  REST_FRAMEWORK = {
      'DEFAULT_AUTHENTICATION_CLASSES': [
          'rest_framework.authentication.TokenAuthentication',
          # 'rest_framework.authentication.SessionAuthentication',  <- remove this
      ],
      # ... other DRF settings
  }
  ```
  This ensures DRF won’t expect CSRF tokens. (SessionAuthentication enforces CSRF validation on requests, by design【34†L39-L46】.) By using only TokenAuthentication, DRF will not perform a CSRF check at the authentication stage. 

- **Exclude API views from Django’s CSRF middleware:** Even if you remove SessionAuth, Django’s global CSRF middleware (if enabled) will still process the request. You have a few ways to handle this:
  - **Use @csrf_exempt decorator** on your API views or view classes. This tells CsrfViewMiddleware to ignore those views. You can apply it to all views under, say, `/api/`. For example, if using function-based views:
    ```python
    from django.views.decorators.csrf import csrf_exempt

    @csrf_exempt
    def my_api_view(request):
        # ... view logic
    ```
    For class-based views or DRF ViewSets, you can apply it via `@method_decorator(csrf_exempt, name='dispatch')` on the class.
  - **Alternatively**, you can subclass DRF’s `SessionAuthentication` to bypass CSRF (DRF has a known snippet called `CsrfExemptSessionAuthentication` if you needed sessions without CSRF). But since you’re only using TokenAuth, simply not using SessionAuth and exempting the view from Csrf middleware is easiest.
  - You could also turn off the CSRF middleware entirely for API routes by adjusting your `MIDDLEWARE` or using Django’s `CsrfViewMiddleware.skip` for certain paths, but the manual exemption is straightforward.

- **Maintain CSRF for non-API parts:** By exempting API endpoints, you still keep CSRF protection on the normal Django site (admin pages, any forms, etc.). This maintains security where it’s needed (cookie-based logins), but avoids CSRF errors on the token-protected API calls. This approach is commonly recommended because token-auth APIs are inherently safe from CSRF【11†L179-L187】【34†L39-L46】.

- **Optional double security:** If you want an extra layer, you *could* issue a CSRF token in addition to the token auth and require both. This would mean implementing the cross-domain CSRF sharing as discussed earlier (setting CSRF cookie and reading it in JS via a custom header). However, most APIs consider this unnecessary. OWASP notes that requiring a custom header (like an API key or token) already significantly mitigates CSRF, since malicious third-party sites cannot fabricate that header on behalf of a user. Thus, it's safe to rely on the token and drop the CSRF requirement for those endpoints in exchange for simplicity and avoiding false rejections.

**Summary:** Configure Django so that API requests authenticated by token are **not subject to CSRF checks**, while still limiting those requests to allowed origins via CORS. This avoids CSRF token errors (since we don’t use cookies for auth there) and maintains security by using the token itself as the anti-forgery mechanism. The combination of proper CORS (to only allow your domains) and token auth means the attack surface for CSRF is minimal.

## 4. Handling HTTP Methods (OPTIONS, GET, POST, etc.) Across Nginx and Django

When enabling CORS and CSRF, special attention must be given to HTTP methods beyond the usual GET/POST, especially the **OPTIONS** method used in CORS preflight requests.

- **CORS Preflight (OPTIONS):** Browsers will automatically send an `OPTIONS` request to your API (to the same URL) before certain requests – specifically if the actual request is not a simple GET/POST or if it includes custom headers like `Authorization`. This is called a preflight. Your server must respond to OPTIONS with a 2xx status and the appropriate CORS headers (`Access-Control-Allow-*`) for the real request to proceed. In the Nginx config above, we handle this by returning 204 directly【19†L303-L310】. If you skip that, ensure that your upstream Django can handle OPTIONS. By default, Django will return HTTP 405 for undefined methods. If you use **django-cors-headers**, its middleware will actually add the CORS headers to the 405, but it won’t inherently return 200. A simpler solution is the Nginx approach shown or using Django REST Framework, which has built-in support for OPTIONS requests on API views (it responds with allowed methods and status 200). In short, make sure OPTIONS isn’t blocked. 

- **Pass through other methods:** Nginx by default will pass GET, POST, PUT, DELETE, etc. to the upstream as long as you don’t have a `limit_except` or a mishandled location. The `proxy_pass` should forward the method as received. Ensure any WAF rules (if present) allow the methods your API uses (we’ll cover WAF specifics in the next section). If your Nginx config uses something like `try_files` or specific handling, double-check that it doesn’t intercept PUT/DELETE unintentionally.

- **Access-Control-Allow-Methods header:** Make sure this header lists **all methods** your front-end might use to call the API. This typically includes `GET, POST, PUT, DELETE, PATCH, OPTIONS` as applicable. The browser will compare the method of the actual request against this list from the preflight response. If, for example, you forget to list `PATCH` and your front-end does a PATCH, the preflight will fail. So keep it in sync with your API. You can include the standard safe methods too; listing `GET, POST, ...` doesn’t do harm for GET requests.

- **Safe vs Unsafe methods and CSRF:** Django (and browsers) consider GET, HEAD, and OPTIONS to be **safe** (read-only) operations that should not have side effects【9†L124-L132】. CSRF protection is only enforced on “unsafe” methods like POST/PUT/DELETE that modify data【9†L126-L134】. This means:
  - You don’t need a CSRF token for GET/OPTIONS requests. Django’s CsrfViewMiddleware won’t complain about those (it only checks POST, etc.). So you only need to worry about CSRF on state-changing operations.
  - However, note that if you are doing something unusual like a GET that changes state (which is bad practice), CSRF wouldn’t protect that – so avoid side effects on GET.
  - HEAD requests (rarely used by front-end JS) are like GET without body; usually no issues there.

- **Django view handling for methods:** If you’re using Django REST Framework, it will automatically handle methods if you use ViewSets or APIView and specify allowed methods. If using function-based views, ensure they properly handle/dispatch methods or use conditionals. If an incoming method is not handled, Django will return 405 Method Not Allowed – ensure that still returns the CORS headers (with the Nginx config, it will, since Nginx adds headers even on error due to `always`). With django-cors-headers, as long as the response is generated, it will append headers as well.

- **Test various requests:** It’s good to test a full flow:
  1. A simple GET from the front-end (should succeed, no preflight).
  2. A POST (should trigger a preflight first because of the `Authorization` header; the OPTIONS should return 204 with allows, then the POST should go through and either succeed or if CSRF is misconfigured, get a 403 – in our setup it should succeed or at least return a meaningful error).
  3. Perhaps a PUT/DELETE if your API supports them.
  4. Also test error cases: e.g., what happens if token is missing or invalid – ensure the error response still includes CORS headers so your JS can handle it. With `always` in Nginx or using the cors middleware, it should.

- **Check Django and Nginx timeouts or size limits:** This is tangential, but sometimes a request like OPTIONS might be handled by Nginx quickly but if not, ensure it’s not being swallowed. Usually not an issue, but just be aware.

By correctly handling all required HTTP methods and especially the **OPTIONS preflight**, you ensure smooth communication between the front-end and back-end without mysterious CORS errors. All methods should be whitelisted in both the Nginx config and allowed by Django’s view logic.

## 5. WAF Rules, Proxies, and Potential Issues (CORS/CSRF Conflicts)

In a setup with Web Application Firewalls (WAFs) or multiple proxy layers, you might encounter additional hurdles. WAFs (such as Nginx with ModSecurity, Cloudflare WAF, AWS WAF, etc.) can sometimes block or alter requests in ways that affect CORS and CSRF. Here’s how to mitigate common issues:

- **OPTIONS method blocking:** Some strict WAF configurations consider uncommon HTTP methods as suspicious. Ensure that your WAF is configured to **allow OPTIONS requests** to your API endpoints. For instance, if using AWS WAF on an API Gateway or ALB, you’d typically have to enable CORS in the gateway or adjust rules accordingly. If using an Nginx-based WAF (like ModSecurity CRS), check that no rule is outright denying OPTIONS. If you find a rule blocking it, create an exception for the path or method.

- **Origin header or cross-domain blocking:** A WAF might have a rule to block requests with an `Origin` header that doesn’t match the site (as a rudimentary CSRF protection). Since our design *intentionally* involves cross-origin requests from known domains, you’ll want to **allow specific Origins** in the WAF policy. For example, some WAFs let you create rules like “if Origin header is not `frontend.example.com` -> block”. Implement such a rule to add another layer of defense beyond CORS. If your WAF doesn’t support dynamic checks, ensure it at least does not block the allowed origin. You may need to add your front-end domain to a whitelist in the WAF configuration.

- **WAF stripping or injecting headers:** Verify that the WAF is not **removing** the `Access-Control-*` headers set by Nginx or Django. In some cases, a proxy might be configured to drop unknown headers. Modern WAFs typically won’t strip standard headers like these, but it’s worth confirming. Similarly, if the WAF or another proxy adds security headers (like CSP, HSTS, etc.), that’s fine – just ensure it doesn’t override our CORS headers. One scenario with AWS’s WAF bot protection is that it introduces a custom header (e.g. `X-Aws-Waf-Token` in requests). If so, you must include that header name in `Access-Control-Allow-Headers` so that preflight doesn’t choke on it【14†L1-L4】. Always update the allowed headers list if you introduce any new custom headers via WAF or otherwise.

- **CSRF-related WAF rules:** Some WAFs (especially with OWASP Core Rule Set) have rules to detect CSRF attacks, such as checking for presence of CSRF tokens in forms or checking the Origin header. If your WAF is flagging legitimate requests, you might see WAF logs indicating something like “CSRF token missing”. In a token-auth API, those rules are not applicable. You should **disable or modify the CSRF rules for the API endpoints**. For example, ModSecurity CRS has an optional rule set for CSRF; you would not include or enable that for API JSON endpoints. If it’s already enabled and causing false positives, exclude the URL path (like `^/api/`) from that rule or disable that rule ID.

- **Multiple Nginx proxies coordinate CORS:** If you have an outer Nginx (WAF) proxy and an inner Nginx (maybe on the app server), decide which one is adding the CORS headers. You typically don’t want both adding potentially different values. It’s simplest to let the outer proxy handle it, as shown. In that case, ensure the inner proxy or application **does not also add** `Access-Control-Allow-Origin` (or if it does via django-cors-headers, it should match exactly the same domain). If using django-cors-headers on the application and not adding in the outer Nginx, then the outer Nginx must be configured to pass through those headers. By default, `proxy_pass` will forward all response headers from the upstream. Just be careful not to have something like `add_header Access-Control-Allow-Origin ...` in both layers with different values. Consistency is key.

- **Testing with WAF:** After deployment behind the WAF, test the full scenario again:
  - Allowed origin request -> should succeed (200 OK, data received).
  - Disallowed origin (you can simulate by changing the Origin header using a tool like curl or a browser plugin) -> should be blocked. The “block” might appear as a 0 response in JS (CORS failure) or the WAF might actively return a 403. If WAF actively returns a forbidden response, ensure that response still has no CORS headers (so the browser just sees it as blocked – either way it’s blocked).
  - If something is blocked incorrectly, check WAF logs. The logs will usually show which rule triggered. Then you can fine-tune: if it’s a false positive, create an exception for that rule on this URL or adjust the rule’s condition.

- **WAF vs CORS error distinction:** Keep in mind, if the WAF blocks a request outright (e.g. via a 403), and if it doesn’t return CORS headers, the browser will report it as a CORS error (since the response is not accessible to JS). This can be confusing. You might see “CORS header missing” but the real cause was WAF. That’s why checking the WAF logs is important when you get a mysterious CORS failure. One strategy to ease debugging is to have the WAF, when blocking, still send an `Access-Control-Allow-Origin: *` (or the specific origin) in the block response. Some WAFs support custom response headers on block pages. This way, the error can be read by the front-end (though you might not want to expose details to potentially malicious origins). This is optional; often just testing and log inspection is enough.

- **Special case – Cloudflare or CDN in front:** If using a CDN or Cloudflare in front of Nginx, ensure its settings aren’t interfering. For Cloudflare, you may need to create a Page Rule to allow the Origin header or disable certain security rules for the API path. Cloudflare by default doesn’t add CORS headers, so you rely on your origin (Nginx) to do it. Cloudflare’s WAF rules might need similar whitelisting as above.

**In summary**, treat the WAF as an additional layer that needs to be configured in harmony with CORS/CSRF settings:
- Whitelist the same domains in WAF rules that you did in Nginx/Django.
- Allow the same headers and methods through.
- Disable or adjust any generic CSRF protections in the WAF (since your app handles it).
- Ensure no conflict or stripping of headers. 

By following these steps and considerations, you’ll have **Nginx** serving the correct CORS headers to only your front-end domains, **Django** accepting those cross-site requests without CSRF issues, and your **WAF** not getting in the way of legitimate traffic. This setup will prevent unauthorized JavaScript on other sites from accessing your Django APIs, while your own application domains will work seamlessly.

## References

- Dynamic CORS in Nginx (whitelisting multiple domains)【19†L292-L300】【2†L196-L203】  
- Django CORS Headers usage and configuration【22†L231-L240】【9†L143-L151】  
- Django CSRF cross-domain trusted origins【11†L179-L187】  
- CSRF requirements for Session vs Token auth (Django/DRF)【34†L41-L46】  
- AWS WAF custom header and CORS allow headers【14†L1-L4】

