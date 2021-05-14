import {Component, OnInit} from '@angular/core';
import {ExperimentViewModelService} from '../../../experiment/services/experiment-view-model.service';
import {MainComponent} from '../../../shared/components/main/main.component';
import {SlimRunner} from '../../../experiment/models';
import {formats} from '../../../../defs';

@Component({
  selector: 'app-runners',
  templateUrl: './runners.component.html',
  styleUrls: ['./runners.component.scss']
})
export class RunnersComponent extends MainComponent implements OnInit {

  runners: SlimRunner[];

  formats = formats;

  get isPageReady(): boolean {
    return !!this.runners;
  }

  constructor(
    private experimentViewModel: ExperimentViewModelService
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.experimentViewModel.runners().subscribe(runners => this.runners = runners)
    );
  }

}
