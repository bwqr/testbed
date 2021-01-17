import { Component, OnInit } from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ExperimentViewModelService} from '../../services/experiment-view-model.service';
import {Experiment} from '../../models';
import {Pagination} from '../../../core/models';

@Component({
  selector: 'app-experiments',
  templateUrl: './experiments.component.html',
  styleUrls: ['./experiments.component.scss']
})
export class ExperimentsComponent extends MainComponent implements OnInit {

  experiments: Pagination<Experiment>;

  constructor(
    private viewModel: ExperimentViewModelService
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.viewModel.experiments().subscribe(experiments => this.experiments = experiments)
    );
  }

}
