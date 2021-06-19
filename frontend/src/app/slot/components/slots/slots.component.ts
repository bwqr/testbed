import {Component, OnInit} from '@angular/core';
import {Slot} from '../../models';
import {SlotViewModelService} from '../../services/slot-view-model.service';
import {MainComponent} from '../../../shared/components/main/main.component';
import {SlimRunner} from '../../../experiment/models';
import {ExperimentViewModelService} from '../../../experiment/services/experiment-view-model.service';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {ActivatedRoute, Router} from '@angular/router';
import {formats} from '../../../../defs';

@Component({
  selector: 'app-slots',
  templateUrl: './slots.component.html',
  styleUrls: ['./slots.component.scss']
})
export class SlotsComponent extends MainComponent implements OnInit {

  slots: [Slot, SlimRunner][];

  runners: SlimRunner[];

  formGroup: FormGroup;

  formats = formats;

  get isPageReady(): boolean {
    return !!this.slots && !!this.runners;
  }

  constructor(
    private viewModel: SlotViewModelService,
    private experimentViewModel: ExperimentViewModelService,
    private formBuilder: FormBuilder,
    private router: Router,
    private activatedRoute: ActivatedRoute,
  ) {
    super();

    this.formGroup = formBuilder.group({
      runnerId: formBuilder.control('', [Validators.required])
    });
  }

  ngOnInit(): void {
    this.subs.add(
      this.viewModel.slots().subscribe(slots => this.slots = slots)
    );

    this.subs.add(
      this.experimentViewModel.runners().subscribe(runners => this.runners = runners)
    );
  }

  redirectToSlotReserve(values): void {
    this.router.navigate(['../slot', values.runnerId, 'reserve'], {
      relativeTo: this.activatedRoute
    }).then();
  }
}
