import {Injectable} from '@angular/core';
import {MainViewModelService} from '../../core/services/main-view-model.service';
import {CacheService} from '../../core/services/cache.service';
import {MainRequestService} from '../../core/services/main-request.service';
import {Observable} from 'rxjs';
import {routes} from '../../routes';

@Injectable({
  providedIn: 'root'
})
export class AdminViewModelService extends MainViewModelService {

  constructor(
    protected cacheService: CacheService,
    protected requestService: MainRequestService
  ) {
    super(cacheService, requestService);
  }

  controllerReceiversValues(controllerId: number): Observable<{ values: number[] | null}> {
    return this.requestService.makeGetRequest(`${routes.experiment.controller}/${controllerId}/values`);
  }
}
