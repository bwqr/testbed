import {Injectable} from '@angular/core';
import {MainViewModelService} from '../../core/services/main-view-model.service';
import {CacheService} from '../../core/services/cache.service';
import {MainRequestService} from '../../core/services/main-request.service';
import {Slot} from '../models';
import {routes} from '../../routes';
import {Observable} from 'rxjs';
import {map} from 'rxjs/operators';
import {SuccessResponse} from '../../core/models';
import {convertDateToLocal, convertDateToServerDate} from '../../helpers';
import {SlimRunner} from '../../experiment/models';

@Injectable({
  providedIn: 'root'
})
export class SlotViewModelService extends MainViewModelService {

  constructor(
    protected cacheService: CacheService,
    protected requestService: MainRequestService
  ) {
    super(cacheService, requestService);
  }

  slots(): Observable<[Slot, SlimRunner][]> {
    return this.requestService.makeGetRequest(routes.slot.slots.root);
  }

  reservedSlots(startAt: Date, runnerId: number, count: number): Observable<Date[]> {
    return this.requestService
      .makeGetRequest(`${routes.slot.slots.reserved}?startAt=${convertDateToServerDate(startAt)}&count=${count}&runnerId=${runnerId}`)
      .pipe(
        map(stringDates => stringDates.map(s => convertDateToLocal(new Date(s))))
      );
  }

  reserveSlot(startAt: Date, runnerId: number): Observable<SuccessResponse> {
    return this.requestService.makePostRequest(routes.slot.slot, {startAt: convertDateToServerDate(startAt), runnerId});
  }
}
