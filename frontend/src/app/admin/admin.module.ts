import {NgModule} from '@angular/core';
import {CommonModule} from '@angular/common';

import {AdminRoutingModule} from './admin-routing.module';
import {RunnerComponent} from './components/runner/runner.component';
import { RunnersComponent } from './components/runners/runners.component';


@NgModule({
  declarations: [RunnerComponent, RunnersComponent],
  imports: [
    CommonModule,
    AdminRoutingModule
  ]
})
export class AdminModule {
}
