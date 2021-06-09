import {Component, OnInit} from '@angular/core';
import {Slot} from '../../models';
import {SlotViewModelService} from '../../services/slot-view-model.service';
import {MainComponent} from '../../../shared/components/main/main.component';

@Component({
  selector: 'app-slots',
  templateUrl: './slot-management.component.html',
  styleUrls: ['./slot-management.component.scss']
})
export class SlotManagementComponent extends MainComponent implements OnInit {

  slots: Slot[];

  constructor(
    private viewModel: SlotViewModelService
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.viewModel.slots().subscribe(slots => this.slots = slots)
    );
  }
}
