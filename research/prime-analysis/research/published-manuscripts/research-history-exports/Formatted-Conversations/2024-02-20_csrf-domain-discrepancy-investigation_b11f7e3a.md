# Csrf Domain Discrepancy Investigation

- **Conversation ID:** `b11f7e3a-f7c4-4385-8232-f5631930e7d3`
- **Date:** 2024-02-20

---

### User (2024-02-20 14:30:24)

At work we are having a problem with our Django websockets after moving them from on premise to Azure - Redis in Azure now. Everything has mostly calmed down now that we've balanced it correctly, but random websocket connections seem to just close with no explaination - the traffic flow is this:
incoming -> nginx -> forward to Azure app container -> app container connects them to media core, which is back on premise - by passing through the same nginx server, and then flowing to our media server that has a java based load balancer on it accepting csrf tokens to it from calls through the proxy... I think that I see 403 errors every so many random packets on the load balancer, but I cannot definitively tie it back to that. What I think is happening is our apps in Azure have a different domain name casey.video instead of caseyvideo.com so is it possibly Django (which our app servers in Azure are running) is connecting back through the nginx proxy to reach resources on the media server (requiring the csrf auth) and it might be passing casey.video now instead of caseyvideo.com - We have CORS allows on our /gateway/ location on our nginx server, but our separate /websocket/ api does not have CORS cross server allowed at all on it - is it possible that for some reason when they log into the application it's getting them through the gateway and generating a csrf token for them, and then passing it along to them later when they use a resource on the media server (through the casey.video frontend in Azure) and the csrf token may be getting rejected and the socket is just closing? Would those 403 errors be indicitive of it? If not, how can I prove or disprove this theory and/or what else should I also consider in your opinion.

### Assistant (2024-02-20 14:31:11)

Your scenario touches on a complex web of interactions between various components in your infrastructure, including Django, Nginx, Azure, on-premise servers, and CSRF token handling. Given the symptoms you're describing, especially the random 403 errors and websocket connections closing without explanation, your hypothesis regarding domain discrepancies affecting CSRF token validation is plausible. Let's tackle this step by step, focusing on potential causes and ways to investigate or resolve the issue.

### Understanding the Problem

1. **Domain Name Discrepancy**: The difference in domain names (`casey.video` vs `caseyvideo.com`) could indeed affect the way CSRF tokens are validated. CSRF tokens are often tied to the domain making the request to prevent Cross-Site Request Forgery attacks. If the domain in the request doesn't match what the server expects, the request could be rejected as a security measure.

2. **CORS and CSRF**: While CORS (Cross-Origin Resource Sharing) and CSRF (Cross-Site Request Forgery) are related to web security, they handle different aspects. CORS restrictions could prevent a browser from making requests to a different domain unless explicitly allowed by the server. However, CSRF protection is about ensuring that requests made to a web application are authorized and originate from the same site to prevent malicious sites from performing actions on behalf of a logged-in user.

3. **403 Errors and Websocket Closures**: A 403 Forbidden error indicates that the server understands the request but refuses to authorize it. If CSRF tokens are not correctly validated, it could lead to such errors. Websockets closing might be a consequence of these unauthorized attempts, especially if the websocket connection relies on initial HTTP handshake requests being authorized.

### Steps to Investigate and Resolve

1. **Verify CSRF Token Domain Scope**: Check how CSRF tokens are being generated and validated in your Django application. Ensure that the tokens are valid for both `casey.video` and `caseyvideo.com` domains if necessary. You might need to adjust settings or logic to accommodate the domain discrepancy.

2. **Examine CORS Configuration**: Although CORS is not directly related to CSRF protection, ensure that your CORS policy on the Nginx server does not inadvertently block or interfere with legitimate requests between your domains or from your Azure-hosted app to your on-premise server.

3. **Debugging and Logging**: Enhance logging at the Nginx proxy, Django application, and the Java-based load balancer to capture detailed information about requests and responses, especially those leading to 403 errors. Look for discrepancies in the request headers, especially the `Origin`, `Referer`, and CSRF token headers.

4. **Simulate the Issue**: Try to replicate the issue in a controlled environment by making requests from `casey.video` to your media server and observing if and when the CSRF token validation fails. This could help confirm if the domain discrepancy is the root cause.

5. **Review Websocket Configuration**: Ensure your websocket configuration on Nginx and within the Django application correctly handles the upgrade from HTTP to Websockets and doesn't drop connections due to misconfiguration or failed security checks.

6. **Cross-Check CSRF Token Handling**: Make sure that the way CSRF tokens are passed and validated in websocket connections (if applicable) is correctly implemented. This might involve reviewing how tokens are attached to websocket initiation requests and how they're validated by the server.

### Additional Considerations

- **Session and Token Lifetimes**: Check if session or token expiration settings could be causing tokens to be considered invalid unexpectedly.
- **Network Issues**: Since part of your infrastructure is on-premise and part is on Azure, network latency or disruptions could also play a role in intermittent issues.

To prove or disprove your theory, start by focusing on logging and simulation tests that directly address the CSRF token validation process across different domains. Adjusting the domain scope for CSRF tokens, if necessary, and ensuring your CORS and CSRF configurations are correctly set up for your architecture, should be your primary focus.

### User (2024-02-21 00:34:12)

if we use django, rest framework, json, knox token csrf authentication passing between these domains and through at least 3 different nginx servers where locations intermittently have cors enabled and doesnt

### Assistant (2024-02-21 00:34:53)

Given the complexity of your setup involving Django, Django REST Framework, JSON Web Tokens (JWT) via Knox for authentication, CSRF tokens, and CORS policies across multiple Nginx servers, managing security and request integrity becomes quite intricate. The issues you're facing could stem from several areas within this stack. Let's delve into a step-by-step approach to address potential causes and solutions, especially focusing on the CSRF token and CORS policy handling amidst this multi-domain, multi-proxy environment.

### 1. Understanding CSRF and CORS in Your Setup

- **CSRF Tokens**: Django uses CSRF tokens to prevent cross-site request forgery attacks. For APIs, especially those authenticated using tokens like Knox, CSRF protection might be handled differently. By default, Django REST Framework may not require CSRF tokens for authenticated API requests since the token-based authentication is considered sufficient. However, if CSRF tokens are being used, their correct handling across domains is crucial.

- **CORS Policies**: CORS policies need to be consistently configured across all Nginx servers to ensure that AJAX requests from the web application (hosted on `casey.video`) to your on-premise backend (`caseyvideo.com`) are not blocked by the browser's same-origin policy. Inconsistencies or misconfigurations in CORS policies could lead to blocked requests or failed preflight checks.

### 2. Diagnosing and Solving CSRF Token Issues

- **Token Validation Across Domains**: Ensure that the CSRF tokens are correctly validated when requests traverse from one domain to another. This might require setting the `CSRF_TRUSTED_ORIGINS` setting in Django to include both `casey.video` and `caseyvideo.com`. 

- **API CSRF Protection**: If your API endpoints are protected by CSRF tokens, confirm that the client-side application correctly sends the CSRF token in the request headers. For non-simple requests, the browser's CORS preflight check must also succeed.

### 3. Ensuring CORS Compatibility

- **Unified CORS Configuration**: Given that CORS is intermittently enabled across Nginx servers, standardize the CORS policy across all servers to ensure consistency. This may involve setting the `Access-Control-Allow-Origin` header to include both domains where necessary and ensuring that preflight requests (`OPTIONS` method) are correctly handled and allowed.

- **CORS and Authentication Tokens**: For endpoints requiring Knox tokens, ensure that the CORS policy does not interfere with the Authorization header. The `Access-Control-Allow-Headers` in your Nginx configuration should include `Authorization` among others needed for your requests.

### 4. Nginx Configuration Review

- Review each Nginx server's configuration to ensure they are correctly forwarding headers and maintaining the integrity of both CSRF and Knox tokens. Misconfiguration here could lead to tokens being dropped or modified.

### 5. Logging and Monitoring

- Implement detailed logging at each stage of your request's journey through the Nginx proxies and the application itself. Log the presence and values of CSRF tokens, Knox tokens, and relevant CORS headers. This can help identify exactly where tokens or headers are being mishandled.

### 6. Testing and Verification

- **Isolate and Test Components**: Try to isolate components and test the flow of requests through each. This can help identify which part of the infrastructure is causing issues with CSRF token validation or CORS handling.
  
- **Cross-Domain Requests**: Simulate cross-domain requests in a controlled environment to ensure that CSRF and Knox tokens are correctly handled and that CORS policies allow these requests.

### 7. Adjusting Django and DRF Settings

- Verify Django's `CSRF_COOKIE_DOMAIN` and `CSRF_TRUSTED_ORIGINS` settings to ensure compatibility with your multi-domain setup.
  
- For Django REST Framework, review the configuration regarding authentication and permission classes to ensure they align with your security requirements, especially in relation to CSRF protection for token-authenticated requests.

### Conclusion

Given the multifaceted nature of the problem, a systematic approach to verifying and adjusting configurations for CSRF and CORS, aligned with thorough testing, is crucial. Ensuring consistency in CORS policies across Nginx servers and validating the handling of CSRF tokens in Django for cross-domain requests will be key steps in resolving the issues you're experiencing.

