import {Injectable} from '@angular/core';
import {HttpEvent, HttpHandler, HttpInterceptor, HttpRequest} from '@angular/common/http';

import {Observable, Subject, throwError} from 'rxjs';
import {catchError} from 'rxjs/operators';
import {Router} from '@angular/router';
import {RequestFailService} from '../services/request-fail.service';
import {ErrorMessage, RetryRequest, ValidationErrorMessage} from '../models';
import {routes} from '../../routes';

@Injectable()
export class HttpConfigInterceptor implements HttpInterceptor {
  loginUrl: string;

  constructor(
    private router: Router,
    private requestFailService: RequestFailService,
    private httpHandler: HttpHandler
  ) {
    // Recursion issue can occur.
    this.requestFailService.retryFailedRequests.subscribe((retryRequest: RetryRequest) => {
        this.intercept(retryRequest.req, this.httpHandler).subscribe(response => {
          retryRequest.subject.next(response);
        });
      }
    );

    this.loginUrl = routes.auth.login;
  }

  intercept(req: HttpRequest<any>, next: HttpHandler): Observable<HttpEvent<any>> {
    const token: string = localStorage.getItem('token');

    req = req.clone({headers: req.headers.set('Authorization', `Bearer ${token}`)});
    req = req.clone({headers: req.headers.set('Accept', 'application/json')});

    // Check if we have already set these headers.
    if (!req.headers.get('Content-Type') && !req.headers.get('enctype')) {
      req = req.clone({headers: req.headers.set('Content-Type', 'application/json')});
    }

    return next.handle(req).pipe(
      catchError(error => this.handlerError(error, req))
    );
  }

  private handlerError(error: any, req: HttpRequest<any>): Observable<ErrorMessage | any> {
    switch (error.status) {
      case 401:
        const navigated = this.router.navigated;
        const notLoginPage = this.router.url.indexOf('/auth/login') === -1;
        const notLoginPopup = this.router.url.indexOf('(auth:popup)') === -1;
        const notLoginRequest = req.url.indexOf(this.loginUrl) === -1;

        const shouldRetryRequest = navigated && notLoginRequest && notLoginPage;
        const shouldNavigateLoginPopup = navigated && notLoginPage && notLoginPopup;
        const shouldNavigateLoginPage = !navigated && notLoginPage;

        if (shouldNavigateLoginPage) {
          this.router.navigate(['auth/login']).then();
        } else if (shouldNavigateLoginPopup) {
          this.router.navigate([{outlets: {auth: ['popup']}}]).then();
        }

        if (shouldRetryRequest) {
          const subject = new Subject();
          setTimeout(() => this.requestFailService.failedRequests.next({req, subject}), 0);
          return subject;
        }

        break;
      default:
        console.error(error);

        // if this is an ErrorMessage
        if (error.error !== null && typeof error.error === 'object' && 'errorCode' in error.error) {
          if ('validationErrors' in error.error) {
            return throwError(new ValidationErrorMessage(error.error));
          } else {
            return throwError(new ErrorMessage(error.error));
          }
        }
    }

    return throwError(error);
  }
}
