import {Injectable} from '@angular/core';
import {CacheService} from './cache.service';
import {MainRequestService} from './main-request.service';
import {HttpParams} from '@angular/common/http';
import {PaginationParams} from '../models';

@Injectable({
  providedIn: 'root'
})
export class MainViewModelService {

  constructor(
    protected cacheService: CacheService,
    protected requestService: MainRequestService
  ) {
  }

  setPaginationParams(params: HttpParams, paginationParams?: PaginationParams): HttpParams {
    if (paginationParams) {
      for (const key in paginationParams) {
        if (paginationParams.hasOwnProperty(key) && paginationParams[key]) {
          params = params.set(key, paginationParams[key]);
        }
      }
    }

    return params;
  }
}
