export const locales = {
  errors: {
    // auth
    invalid_credentials: $localize`:@@errors.invalid_credentials:Invalid Credentials`,
    user_exists: $localize`:@@errors.user_exists:This email is already in use`,
    not_verified: $localize`:@@errors.not_verified:Your account is not verified. Please verify it via email`,
    // experiment
    not_allowed_to_run_for_slot: $localize`:@@errors.not_allowed_to_run_for_slot:This slot does not belongs to you. You cannot run an experiment in this slot. Please reserve a slot`,
    // slot
    invalid_slot_interval: $localize`:@@errors.invalid_slot_interval:Given slot interval is invalid`,
    validation_errors: $localize`:@@errors.validation_errors:There are some validation errors`,
    validationErrors: {}
  },
  auth: {
    errors: {
      emailNotFound: $localize`:@@auth.errors.emailNotFound:Email address could not be found`,
      invalidOrExpiredToken: $localize`:@@auth.errors.invalidOrExpiredToken:Token is not valid or expired`
    }
  }
};
