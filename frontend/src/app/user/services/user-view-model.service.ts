import {Injectable} from '@angular/core';
import {CacheService} from '../../core/services/cache.service';
import {MainRequestService} from '../../core/services/main-request.service';
import {MainViewModelService} from '../../core/services/main-view-model.service';
import {Observable, Subject} from 'rxjs';
import {User} from '../models';
import {routes} from '../../routes';
import {SuccessResponse} from '../../core/models';

@Injectable({
  providedIn: 'root'
})
export class UserViewModelService extends MainViewModelService {

  $user = new Subject<User>();

  constructor(
    protected cacheService: CacheService,
    protected requestService: MainRequestService
  ) {
    super(cacheService, requestService);
  }

  user(): Observable<User> {
    return this.cacheService.get('user.user', this.requestService.makeGetRequest(routes.user.profile));
  }

  updateProfile(firstName: string, lastName: string): Observable<SuccessResponse> {
    return this.requestService.makePutRequest(routes.user.profile, {firstName, lastName});
  }

  updatePassword(password: string): Observable<SuccessResponse> {
    return this.requestService.makePutRequest(routes.user.password, {password});
  }
}
