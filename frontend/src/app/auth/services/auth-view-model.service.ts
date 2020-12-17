import {Injectable} from '@angular/core';
import {MainViewModelService} from '../../core/services/main-view-model.service';
import {CacheService} from '../../core/services/cache.service';
import {MainRequestService} from '../../core/services/main-request.service';
import {Observable} from 'rxjs';
import {SuccessResponse, TokenResponse} from '../../core/models';
import {routes} from '../../routes';
import {AuthService} from './auth.service';
import {tap} from 'rxjs/operators';

@Injectable({
  providedIn: 'root'
})
export class AuthViewModelService extends MainViewModelService {

  constructor(
    protected cacheService: CacheService,
    protected requestService: MainRequestService,
    private service: AuthService
  ) {
    super(cacheService, requestService);
  }

  login(email: string, password: string): Observable<TokenResponse> {
    return this.requestService.makePostRequest(routes.auth.login, {email, password}).pipe(
      tap(token => this.service.setToken(token.token)),
    );
  }

  signUp(firstName: string, lastName: string, email: string, password: string): Observable<SuccessResponse> {
    return this.requestService.makePostRequest(routes.auth.signUp, {firstName, lastName, email, password});
  }

  logout(): void {
    this.cacheService.clear();
    this.service.setToken(null);
  }

  verifyAccount(token: string): Observable<SuccessResponse> {
    return this.requestService.makePostRequest(routes.auth.verifyAccount, {token});
  }

  resetPassword(token: string, password: any): Observable<SuccessResponse> {
    return this.requestService.makePutRequest(routes.auth.resetPassword, {token, password});
  }

  forgotPassword(email: string): Observable<SuccessResponse> {
    return this.requestService.makePostRequest(routes.auth.forgotPassword, {email});
  }
}
