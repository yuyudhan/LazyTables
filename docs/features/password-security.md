# Password Security Features

LazyTables provides multiple secure options for managing database passwords in your connections.

## Password Storage Options

When creating or editing a connection, you can choose from three password storage methods:

### 1. Plain Text (Default)
- Simple password entry
- Password is stored in the configuration file
- Suitable for development environments

### 2. Environment Variable
- Reference passwords from environment variables
- Enter the variable name (e.g., `DB_PASSWORD`)
- LazyTables will read the password from the environment at connection time
- Recommended for production environments

### 3. Encrypted (AES-256)
- Passwords are encrypted using AES-256-GCM encryption
- You provide an encryption key that is never stored
- Optional hint can be saved to remind you of the key
- Most secure option for local password storage

## How to Use

1. **Create a new connection**: Press the appropriate key to open the connection modal
2. **Navigate with Tab**: Use Tab and Shift+Tab to move between fields
3. **Select Password Storage Type**: Use arrow keys (↑↓) when on the "Password Storage Type" field
4. **Enter appropriate details**:
   - **Plain Text**: Just enter your password
   - **Environment Variable**: Enter the variable name (without $)
   - **Encrypted**: Enter password, encryption key, and optional hint

## Security Notes

- Encryption keys are never stored - you must remember them
- Environment variables are resolved at connection time
- Encrypted passwords use industry-standard AES-256-GCM encryption
- Key derivation uses Argon2 for additional security

## Examples

### Using Environment Variable
```bash
export DB_PASSWORD="my-secure-password"
```
Then in LazyTables, select "Environment Variable" and enter `DB_PASSWORD`

### Using Encrypted Password
- Password: `my-database-password`
- Encryption Key: `my-secret-key-123`
- Hint: `favorite number + pet name`

The password will be encrypted and stored securely. You'll need to provide the encryption key when connecting.