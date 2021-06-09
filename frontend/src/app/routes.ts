import {environment} from '../environments/environment';

const experiment = environment.apiEndpoint + '/experiment';
const user = environment.apiEndpoint + '/user';
const auth = environment.apiEndpoint + '/auth';
const slot = environment.apiEndpoint + '/slot';

export const routes = {
  user: {
    profile: user + '/profile',
    password: user + '/password'
  },
  experiment: {
    runners: experiment + '/runners',
    runner: experiment + '/runner',
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
  },
  slot: {
    slots: {
      root: slot + '/slots',
      reserved: slot + '/slots/reserved'
    },
    slot: slot + '/slot'
  }
};
