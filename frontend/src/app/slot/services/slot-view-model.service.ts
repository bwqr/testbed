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
import {SlimController} from '../../experiment/models';

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

  private static mapSlot(slot: Slot): Slot {
    slot.startAt = convertDateToLocal(slot.startAt);
    slot.endAt = convertDateToLocal(slot.endAt);
    slot.createdAt = convertDateToLocal(slot.createdAt);
    slot.updatedAt = convertDateToLocal(slot.updatedAt);

    return slot;
  }

  slots(): Observable<[Slot, SlimController][]> {
    return this.requestService.makeGetRequest(routes.slot.slots.root).pipe(
      map(slots => slots.map(ss => {
          ss[0] = SlotViewModelService.mapSlot(ss[0]);
          return ss;
        })
      )
    );
  }

  reservedSlots(startAt: Date, controllerId: number, count: number): Observable<Date[]> {
    return this.requestService
      .makeGetRequest(`${routes.slot.slots.reserved}?startAt=${convertDateToServerDate(startAt)}&count=${count}&controllerId=${controllerId}`)
      .pipe(
        map(stringDates => stringDates.map(s => convertDateToLocal(new Date(s))))
      );
  }

  reserveSlot(startAt: Date, controllerId: number): Observable<SuccessResponse> {
    return this.requestService.makePostRequest(routes.slot.slot, {startAt: convertDateToServerDate(startAt), controllerId});
  }

  cancelSlot(id: number): Observable<SuccessResponse> {
    return this.requestService.makeDeleteRequest(`${routes.slot.slot}/${id}`);
  }
}
