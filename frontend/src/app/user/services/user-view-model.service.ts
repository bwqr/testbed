import {Injectable} from '@angular/core';
import {CacheService} from '../../core/services/cache.service';
import {MainRequestService} from '../../core/services/main-request.service';
import {MainViewModelService} from '../../core/services/main-view-model.service';
import {Observable} from 'rxjs';
import {User} from '../models';
import {routes} from '../../routes';

@Injectable({
  providedIn: 'root'
})
export class UserViewModelService extends MainViewModelService {

  constructor(
    protected cacheService: CacheService,
    protected requestService: MainRequestService
  ) {
    super(cacheService, requestService);
  }

  profile(): Observable<User> {
    return this.requestService.makeGetRequest(routes.user.profile);
  }
}
