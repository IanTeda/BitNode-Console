# Routing

The frontend web application has the following routing structure:

```bash
Protected Routes:
/                 # Root route will redirect to /dashboard
├── /dashboard
├── /node
├── /network
├── /logs
├── /settings
└── /<any other route> # Any other route not defined will redirect to /dashboard if the user is authenticated or login if not

Public Routes:
└── /auth
  |── /login
  |── /logged-out
  ├── /verifying
  └── /<any other route> # Any other route not defined will redirect to /login
```
