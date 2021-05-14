import {NgModule} from '@angular/core';
import {RouterModule, Routes} from '@angular/router';
import {RunnerComponent} from './components/runner/runner.component';
import {RunnersComponent} from './components/runners/runners.component';

const routes: Routes = [
  {path: 'runners', component: RunnersComponent},
  {path: 'runner/:id', component: RunnerComponent}
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class AdminRoutingModule {
}
