import {Component, OnInit} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {SlotViewModelService} from '../../services/slot-view-model.service';
import {ActivatedRoute} from '@angular/router';
import {combineAll, finalize, switchMap} from 'rxjs/operators';
import {formats} from '../../../../defs';
import {BehaviorSubject, combineLatest, Subject} from 'rxjs';

@Component({
  selector: 'app-slot-reserve',
  templateUrl: './slot-reserve.component.html',
  styleUrls: ['./slot-reserve.component.scss']
})
export class SlotReserveComponent extends MainComponent implements OnInit {
  reserved: { reserved: boolean; date: Date }[];

  formats = formats;

  runnerId: number;

  filterDate = new Date();

  trigger = new BehaviorSubject(null);

  get isPageReady(): boolean {
    return !!this.reserved;
  }

  constructor(
    private viewModel: SlotViewModelService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      combineLatest([this.activatedRoute.params, this.trigger]).pipe(
        switchMap(([params, _]) => {
          this.runnerId = parseInt(params.runnerId, 0);
          return this.viewModel.reservedSlots(this.filterDate, params.runnerId, 24 - (new Date()).getHours());
        })
      ).subscribe(dates => {
        this.reserved = [];
        const filter = new Date(this.filterDate.getTime());
        let currentDate = dates.shift();

        for (let i = (new Date()).getHours(); i < 24; i++) {
          const date = new Date(filter.getTime());
          date.setMinutes(0);
          date.setSeconds(0);

          if (currentDate && currentDate.getHours() === filter.getHours()) {
            this.reserved.push({reserved: true, date});
            currentDate = dates.shift();
          } else {
            this.reserved.push({reserved: false, date});
          }

          filter.setHours(filter.getHours() + 1);
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
    this.trigger.next();
  }

  nextDay(): void {
    this.filterDate = new Date(this.filterDate.getTime() + 60 * 60 * 24 * 1000);
    this.trigger.next();
  }
}
