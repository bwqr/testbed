import {Component, OnInit} from '@angular/core';
import {Slot} from '../../models';
import {SlotViewModelService} from '../../services/slot-view-model.service';
import {MainComponent} from '../../../shared/components/main/main.component';
import {SlimController} from '../../../experiment/models';
import {ExperimentViewModelService} from '../../../experiment/services/experiment-view-model.service';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {ActivatedRoute, Router} from '@angular/router';
import {formats} from '../../../../defs';
import {MainService} from '../../../core/services/main.service';
import {catchError, finalize} from 'rxjs/operators';
import {ErrorMessage} from '../../../core/models';

@Component({
  selector: 'app-slots',
  templateUrl: './slots.component.html',
  styleUrls: ['./slots.component.scss']
})
export class SlotsComponent extends MainComponent implements OnInit {

  slots: [Slot, SlimController][];

  controllers: SlimController[];

  formGroup: FormGroup;

  formats = formats;

  get isPageReady(): boolean {
    return !!this.slots && !!this.controllers;
  }

  constructor(
    private viewModel: SlotViewModelService,
    private experimentViewModel: ExperimentViewModelService,
    private formBuilder: FormBuilder,
    private router: Router,
    private activatedRoute: ActivatedRoute,
    private service: MainService,
  ) {
    super();

    this.formGroup = this.formBuilder.group({
      controllerId: formBuilder.control('', [Validators.required])
    });
  }

  ngOnInit(): void {
    this.subs.add(
      this.viewModel.slots().subscribe(slots => this.slots = slots)
    );

    this.subs.add(
      this.experimentViewModel.controllers().subscribe(controllers => this.controllers = controllers)
    );
  }

  redirectToSlotReserve(values: {controllerId: number}): void {
    this.router.navigate(['../slot', values.controllerId, 'reserve'], {
      relativeTo: this.activatedRoute
    }).then();
  }

  cancelSlot(id: number): void {
    if (this.isInProcessingState) {
      return;
    }

    this.enterProcessingState();

    this.subs.add(
      this.viewModel.cancelSlot(id).pipe(
        finalize(() => this.leaveProcessingState()),
        catchError(errorMessage => {
          if (errorMessage instanceof ErrorMessage) {
            this.service.alertFail(errorMessage.message.localized);
          }

          return Promise.reject(errorMessage);
        })
      )
        .subscribe(() => {
          this.service.alertSuccess('Slot is cancelled');
          const index = this.slots.findIndex(s => s[0].id === id);

          if (index > -1) {
            this.slots.splice(index, 1);
          }
        })
    );
  }
}
