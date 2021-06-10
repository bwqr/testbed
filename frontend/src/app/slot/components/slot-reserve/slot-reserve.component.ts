import {Component, OnInit} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {SlotViewModelService} from '../../services/slot-view-model.service';
import {ActivatedRoute} from '@angular/router';
import {finalize, switchMap} from 'rxjs/operators';
import {formats} from '../../../../defs';
import {BehaviorSubject, combineLatest} from 'rxjs';

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
  isFilterToday: boolean;

  trigger = new BehaviorSubject(null);

  get isPageReady(): boolean {
    return !!this.reserved;
  }

  constructor(
    private viewModel: SlotViewModelService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
    this.filterDate = new Date();
    // set filterDate to start of the today
    this.filterDate.setSeconds(0);
    this.filterDate.setMinutes(0);
    this.filterDate.setHours(0);
    this.isFilterToday = true;
  }

  ngOnInit(): void {
    this.subs.add(
      combineLatest([this.activatedRoute.params, this.trigger]).pipe(
        switchMap(([params, _]) => {
          this.runnerId = parseInt(params.runnerId, 0);
          const today = new Date();
          today.setSeconds(0);
          today.setMinutes(0);
          today.setHours(0);

          this.isFilterToday = Math.floor(this.filterDate.getTime() / (60 * 60 * 24)) === Math.floor(today.getTime() / (60 * 60 * 24));

          return this.viewModel.reservedSlots(this.filterDate, this.runnerId, 24);
        })
      ).subscribe(dates => {
        this.reserved = [];
        let currentDate = dates.shift();

        for (let i = 0; i < 24; i++) {
          const date = new Date(this.filterDate.getTime());
          date.setMinutes(0);
          date.setSeconds(0);
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
}
