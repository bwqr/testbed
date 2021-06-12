import {Injectable} from '@angular/core';
import {PaginationParams} from '../models';
import {Params, Router} from '@angular/router';

@Injectable({
  providedIn: 'root'
})
export class PaginationService {

  static DEFAULT_PER_PAGE = 10;
  static DEFAULT_PAGE = 1;

  constructor(
    private router: Router
  ) {
  }

  getPaginationFromParams(params: Params): PaginationParams {
    return {
      perPage: parseInt(params.perPage || PaginationService.DEFAULT_PER_PAGE, 10),
      page: parseInt(params.page || PaginationService.DEFAULT_PAGE, 10)
    };
  }

  getPaginationFromEvent(event: /*PageChangedEvent*/ any): PaginationParams {
    return {
      perPage: event.itemsPerPage,
      page: event.page
    };
  }

  pageChanged(event: /*PageChangedEvent*/ any): void {
    this.router.navigate([], {
      queryParams: this.getPaginationFromEvent(event),
      queryParamsHandling: 'merge'
    }).then();
  }
}
