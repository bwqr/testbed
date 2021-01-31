import {Injectable} from '@angular/core';
import {MainViewModelService} from '../../core/services/main-view-model.service';
import {CacheService} from '../../core/services/cache.service';
import {MainRequestService} from '../../core/services/main-request.service';
import {Observable} from 'rxjs';
import {Experiment, Job, SlimJob, SlimRunner} from '../models';
import {routes} from '../../routes';
import {Pagination, PaginationParams, SuccessResponse} from '../../core/models';
import {HttpParams} from '@angular/common/http';

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

  runners(): Observable<SlimRunner[]> {
    return this.cacheService.get('experiment.runners', this.requestService.makeGetRequest(routes.experiment.runners));
  }

  runExperiment(experimentId: number, runnerId: number): Observable<SuccessResponse> {
    return this.requestService.makePostRequest(`${routes.experiment.experiment}/${experimentId}/run/${runnerId}`, {});
  }

  experimentJobs(experimentId: number, paginationParams?: PaginationParams): Observable<Pagination<[SlimJob, SlimRunner]>> {
    const params = this.setPaginationParams(new HttpParams(), paginationParams);

    return this.requestService.makeGetRequestWithParams(
      `${routes.experiment.experiment}/${experimentId}/jobs`,
      params
    );
  }

  job(id: number): Observable<[Job, SlimRunner]> {
    return this.requestService.makeGetRequest(`${routes.experiment.job}/${id}`);
  }

  experiments(paginationParams?: PaginationParams): Observable<Pagination<Experiment>> {
    const params = this.setPaginationParams(new HttpParams(), paginationParams);

    return this.requestService.makeGetRequestWithParams(routes.experiment.experiments, params);
  }

  experiment(id: number): Observable<Experiment> {
    return this.requestService.makeGetRequest(`${routes.experiment.experiment}/${id}`);
  }

  storeExperiment(name: string): Observable<SuccessResponse> {
    return this.requestService.makePostRequest(routes.experiment.experiment, {name});
  }

  updateExperiment(id: number, code: string): Observable<SuccessResponse> {
    return this.requestService.makePutRequest(`${routes.experiment.experiment}/${id}/code`, {code});
  }
}
