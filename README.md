# Axum OAuth Sample

An axum example of authentication using oauth.

<https://github.com/Neo-Ciber94/axum-oauth-sample/assets/7119315/340580d2-c221-4f09-82f0-f343b50d4b96>

This example uses:

- Axum
- Askama
- Sqlx **(with SQlite)**
- TailwindCSS

And have oauth authentication for these providers:

- Google
- Github
- Discord

## Missing features

- Refresh tokens
- Token revocation

## How to run

### Prerequisites

- [node >= 18](https://nodejs.org/en)
- [sqlx-cli](https://crates.io/crates/sqlx-cli)

1. Install dependencies

```bash
cargo install
pnpm install # Or remove pnpm-lock.yaml and run `npm install`
```

2. Create database and run migrations

```bash
mkdir data
sqlx database create
sqlx migrate run
```

3. Run

```bash
npm run tw:watch
cargo run # In other shell
```

## Docker

Build the image:

```bash
docker build . -t axum-oauth
```

```bash
docker run -dp 5000:5000 -e HOST="0.0.0.0" -e PORT=5000 -e BASE_URL="http://localhost:5000" --env-file=.env.docker axum-oauth
```

> This require create a `.env.docker` file with similar to `.env.sample`

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
