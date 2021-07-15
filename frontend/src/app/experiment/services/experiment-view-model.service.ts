import {Injectable} from '@angular/core';
import {MainViewModelService} from '../../core/services/main-view-model.service';
import {CacheService} from '../../core/services/cache.service';
import {MainRequestService} from '../../core/services/main-request.service';
import {Observable} from 'rxjs';
import {Experiment, Job, SlimJob, SlimRunner} from '../models';
import {routes} from '../../routes';
import {Pagination, PaginationParams, SuccessResponse} from '../../core/models';
import {HttpParams} from '@angular/common/http';
import {map} from 'rxjs/operators';
import {convertDateToLocal} from '../../helpers';

@Injectable({
  providedIn: 'root'
})
export class ExperimentViewModelService extends MainViewModelService {

  constructor(
    protected cacheService: CacheService,
    protected requestService: MainRequestService,
  ) {
    super(cacheService, requestService);
  }

  runners(): Observable<SlimRunner[]> {
    return this.cacheService.get('experiment.runners', this.requestService.makeGetRequest(routes.experiment.runners));
  }

  runner(id: number): Observable<SlimRunner> {
    return this.cacheService.get(`experiment.runner.${id}`, this.requestService.makeGetRequest(`${routes.experiment.runner}/${id}`));
  }

  runExperiment(experimentId: number, runnerId: number): Observable<Job> {
    return this.requestService.makePostRequest(`${routes.experiment.experiment}/${experimentId}/run/${runnerId}`, {}).pipe(
      map(j => {
        j.createdAt = convertDateToLocal(j.createdAt);
        j.updatedAt = convertDateToLocal(j.updatedAt);
        return j;
      })
    );
  }

  experimentJobs(experimentId: number, paginationParams?: PaginationParams): Observable<Pagination<[SlimJob, SlimRunner]>> {
    const params = this.setPaginationParams(new HttpParams(), paginationParams);

    return this.requestService.makeGetRequestWithParams(
      `${routes.experiment.experiment}/${experimentId}/jobs`,
      params
    );
  }

  job(id: number): Observable<[Job, SlimRunner]> {
    return this.requestService.makeGetRequest(`${routes.experiment.job}/${id}`).pipe(
      map(js => {
        js[0].createdAt = convertDateToLocal(js[0].createdAt);
        js[0].updatedAt = convertDateToLocal(js[0].updatedAt);
        return js;
      })
    );
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

  abortRunningJob(id: number): Observable<SuccessResponse> {
    return this.requestService.makeDeleteRequest(`${routes.experiment.job}/${id}/abort`);
  }
}
