import {NgModule} from '@angular/core';
import {RouterModule, Routes} from '@angular/router';
import {ExperimentsComponent} from './components/experiments/experiments.component';
import {ExperimentComponent} from './components/experiment/experiment.component';
import {ExperimentAddComponent} from './components/experiment-add/experiment-add.component';

const routes: Routes = [
  {path: 'experiments', component: ExperimentsComponent},
  {path: 'experiment/:id', component: ExperimentComponent},
  {path: 'add', component: ExperimentAddComponent},
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class ExperimentRoutingModule {
}
