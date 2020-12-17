import {Injectable} from '@angular/core';
import {Subject} from 'rxjs';
import {RetryRequest} from '../models';


/**
 * This service responsibility is to set a bridge between Auth::LoginPopupComponent and Core::HttpConfigInterceptor.
 * When an user gets 401 error response from server, interceptor will redirect to LoginPopup as outlet named auth.
 * While redirecting,it will dispatch a new RetryRequest object into failedRequests subject. This dispatching will
 * be caught by LoginPopup, and after successful authorization, it will dispatch the requests into retryFailedRequests,
 * which is listened by interceptor. Interceptor will handle the dispatched requests.
 *
 * The issue is that, there can be some non breaking recursion problem. If server starts to give 401 responses
 * indefinitely, our application will go into an recursion and be in the undefined state. A retry count can be
 * added into RetryRequest, however this will require to change the interceptor's intercept function, which
 * will need the RetryRequest object and pass into handleError function.
 */
@Injectable({
  providedIn: 'root'
})
export class RequestFailService {

  failedRequests = new Subject<RetryRequest>();

  retryFailedRequests = new Subject<RetryRequest>();

  constructor() {
  }
}
