import {environment} from '../environments/environment';

const experiment = environment.apiEndpoint + '/experiment';
const user = environment.apiEndpoint + '/user';
const auth = environment.apiEndpoint + '/auth';

export const routes = {
  user: {
    profile: user + '/profile',
    password: user + '/password'
  },
  experiment: {
    runners: experiment + '/runners',
    experiments: experiment + '/experiments',
    experiment: experiment + '/experiment',
    job: experiment + '/job'
  },
  auth: {
    login: auth + '/login',
    signUp: auth + '/sign-up',
    verifyAccount: auth + '/verify-account',
    resetPassword: auth + '/reset-password',
    forgotPassword: auth + '/forgot-password'
  }
};
