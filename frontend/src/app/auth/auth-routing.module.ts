import {NgModule} from '@angular/core';
import {RouterModule, Routes} from '@angular/router';
import {LoginComponent} from './components/login/login.component';
import {SignUpComponent} from './components/sign-up/sign-up.component';
import {LogoutComponent} from './components/logout/logout.component';
import {VerifyAccountComponent} from './components/verify-account/verify-account.component';
import {ResetPasswordComponent} from './components/reset-password/reset-password.component';
import {ForgotPasswordComponent} from './components/forgot-password/forgot-password.component';
import {LoginDialogComponent} from './dialogs/login-dialog/login-dialog.component';
import {AuthComponent} from './components/auth/auth.component';

const routes: Routes = [
  {
    path: 'auth', component: AuthComponent, children: [
      {path: 'login', component: LoginComponent},
      {path: 'sign-up', component: SignUpComponent},
      {path: 'logout', component: LogoutComponent},
      {path: 'verify-account', component: VerifyAccountComponent},
      {path: 'reset-password', component: ResetPasswordComponent},
      {path: 'forgot-password', component: ForgotPasswordComponent},
    ]
  },
  {path: 'popup', component: LoginDialogComponent, outlet: 'auth'},
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AuthRoutingModule {
}
