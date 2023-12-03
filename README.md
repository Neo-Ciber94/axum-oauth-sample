# Axum OAuth Sample

An axum example of authentication using oauth.

This example uses:

- Axum
- Askama
- Sqlx **(with SQlite)**
- TailwindCSS

And have oauth authentication for these providers:

- Google
- Github
- Discord

## Authentication workflow

### Login

```mermaid
graph TD
  A[Login] -->|"1. Request /api/auth/{provider}/login"| B[Redirect to Provider]
  B -->|"2. Redirect to OAuth provider"| C[Provider Authorization Page]
  C -->|"3. User authorizes"| D["Redirect to /api/auth/{provider}/callback"]
  D -->|"4. Exchange code for token"| E[Token Response]
  E -->|"5. Request user info"| F[Get user information]
  F -->|"6. Create or retrieve user"| G[Database - Create/Retrieve User]
  G -->|"7. Create user session"| H[Database - Create User Session]
  H -->|"8. Remove cookies"| I[Remove CSRF and Code Verifier Cookies]
  I -->|"9. Set session cookie"| J[Set Session Cookie]
  J -->|"10. Redirect to /"| K[Redirect to Home]
```

### Logout

```mermaid
graph TD
  L[Logout] -->|"1. Request /api/auth/logout"| M[Check session cookie]
  M -->|"2. Delete user session"| N[Database - Delete User Session]
  N -->|"3. Remove session cookie"| O[Remove Session Cookie]
  O -->|"4. Redirect to /"| P[Redirect to Home]
```

## Missing features

- Refresh tokens
- Token revocation
