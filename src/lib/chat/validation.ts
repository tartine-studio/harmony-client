export interface Credentials {
  serverUrl: string;
  username: string;
  email: string;
  password: string;
}

export type Field = keyof Credentials;
export type FieldError =
  | "validation.serverRequired"
  | "validation.usernameLength"
  | "validation.emailInvalid"
  | "validation.passwordLength"
  | "validation.passwordRequired";

const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/** Mirrors the server rules so errors show before any request. */
export function validateCredentials(
  values: Credentials,
  mode: "login" | "register",
): Partial<Record<Field, FieldError>> {
  const errors: Partial<Record<Field, FieldError>> = {};
  if (!values.serverUrl.trim()) errors.serverUrl = "validation.serverRequired";
  if (!EMAIL.test(values.email.trim())) errors.email = "validation.emailInvalid";
  if (mode === "register") {
    const len = Array.from(values.username.trim()).length;
    if (len < 2 || len > 32) errors.username = "validation.usernameLength";
    if (values.password.length < 8 || values.password.length > 128) errors.password = "validation.passwordLength";
  } else if (!values.password) {
    errors.password = "validation.passwordRequired";
  }
  return errors;
}
