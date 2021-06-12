import {Component, OnInit} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {SlotViewModelService} from '../../services/slot-view-model.service';
import {ActivatedRoute} from '@angular/router';
import {finalize, switchMap} from 'rxjs/operators';
import {formats} from '../../../../defs';
import {BehaviorSubject, combineLatest} from 'rxjs';
import {MainService} from '../../../core/services/main.service';

@Component({
  selector: 'app-slot-reserve',
  templateUrl: './slot-reserve.component.html',
  styleUrls: ['./slot-reserve.component.scss']
})
export class SlotReserveComponent extends MainComponent implements OnInit {
  reserved: { reserved: boolean; date: Date }[];

  formats = formats;

  runnerId: number;

  filterDate: Date;
  isFilterDateToday: boolean;

  trigger = new BehaviorSubject(null);

  get isPageReady(): boolean {
    return !!this.reserved;
  }

  constructor(
    private viewModel: SlotViewModelService,
    private activatedRoute: ActivatedRoute,
  ) {
    super();
    this.filterDate = this.startOfDay((new Date()).getTime());
    this.isFilterDateToday = true;
  }

  ngOnInit(): void {
    this.subs.add(
      combineLatest([this.activatedRoute.params, this.trigger]).pipe(
        switchMap(([params, _]) => {
          this.runnerId = parseInt(params.runnerId, 10);
          const now = new Date();
          const startOfToday = this.startOfDay(now.getTime());

          this.isFilterDateToday = this.filterDate.getTime() === startOfToday.getTime();

          if (this.isFilterDateToday) {
            const count = 24 - now.getHours();
            now.setMilliseconds(0);
            now.setSeconds(0);
            now.setMinutes(0);
            return this.viewModel.reservedSlots(now, this.runnerId, count);
          }

          return this.viewModel.reservedSlots(this.filterDate, this.runnerId, 24);
        })
      ).subscribe(dates => {
        this.reserved = [];
        let currentDate = dates.shift();

        const startHour = this.isFilterDateToday ? (new Date()).getHours() : 0;

        for (let i = startHour; i < 24; i++) {
          const date = this.startOfDay(this.filterDate.getTime());
          date.setHours(i);

          if (currentDate && currentDate.getHours() === date.getHours()) {
            this.reserved.push({reserved: true, date});
            currentDate = dates.shift();
          } else {
            this.reserved.push({reserved: false, date});
          }
        }
      })
    );
  }

  reserveSlot(res: { reserved: boolean; date: Date }): void {
    this.enterProcessingState();
    this.subs.add(
      this.viewModel.reserveSlot(res.date, this.runnerId)
        .pipe(
          finalize(() => this.leaveProcessingState())
        )
        .subscribe(() => res.reserved = true)
    );
  }

  previousDay(): void {
    this.filterDate = new Date(this.filterDate.getTime() - 60 * 60 * 24 * 1000);
    this.trigger.next(null);
  }

  nextDay(): void {
    this.filterDate = new Date(this.filterDate.getTime() + 60 * 60 * 24 * 1000);
    this.trigger.next(null);
  }

  startOfDay(time: number): Date {
    const date = new Date(time);
    date.setMilliseconds(0);
    date.setSeconds(0);
    date.setMinutes(0);
    date.setHours(0);

    return date;
  }
}
