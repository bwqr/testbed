import {NgModule} from '@angular/core';
import {CommonModule} from '@angular/common';
import {ExperimentRoutingModule} from './experiment-routing.module';
import { ExperimentsComponent } from './components/experiments/experiments.component';
import {CoreModule} from '../core/core.module';
import {SharedModule} from '../shared/shared.module';
import { ExperimentComponent } from './components/experiment/experiment.component';
import { ExperimentAddComponent } from './components/experiment-add/experiment-add.component';
import { JobComponent } from './components/job/job.component';


@NgModule({
  declarations: [ExperimentsComponent, ExperimentComponent, ExperimentAddComponent, JobComponent],
  imports: [
    CommonModule,
    ExperimentRoutingModule,
    CoreModule,
    SharedModule
  ]
})
export class ExperimentModule {
}
