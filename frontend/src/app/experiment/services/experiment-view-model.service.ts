import {Injectable} from '@angular/core';
import {MainViewModelService} from '../../core/services/main-view-model.service';
import {CacheService} from '../../core/services/cache.service';
import {MainRequestService} from '../../core/services/main-request.service';
import {Observable} from 'rxjs';
import {Experiment} from '../models';
import {routes} from '../../routes';
import {Pagination, SuccessResponse} from '../../core/models';

@Injectable({
  providedIn: 'root'
})
export class ExperimentViewModelService extends MainViewModelService {

  constructor(
    protected cacheService: CacheService,
    protected requestService: MainRequestService
  ) {
    super(cacheService, requestService);
  }

  experiments(): Observable<Pagination<Experiment>> {
    return this.requestService.makeGetRequest(routes.experiment.experiments);
  }

  experiment(id: number): Observable<Experiment> {
    return this.requestService.makeGetRequest(`${routes.experiment.experiment}/${id}`);
  }

  storeExperiment(name: string): Observable<SuccessResponse> {
    return this.requestService.makePostRequest(routes.experiment.experiment, {name});
  }
}
