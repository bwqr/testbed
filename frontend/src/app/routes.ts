import {environment} from '../environments/environment';

export const routes = {
  user: {
    profile: environment.apiEndpoint + '/user/profile',
  },
  auth: {
    login: environment.apiEndpoint + '/auth/login',
    signUp: environment.apiEndpoint + '/auth/sign-up',
    verifyAccount: environment.apiEndpoint + '/auth/verify-account',
    resetPassword: environment.apiEndpoint + '/auth/reset-password',
    forgotPassword: environment.apiEndpoint + '/auth/forgot-password'
  }
};
