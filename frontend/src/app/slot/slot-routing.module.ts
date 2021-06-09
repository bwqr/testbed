import { NgModule } from '@angular/core';
import { Routes, RouterModule } from '@angular/router';
import {SlotManagementComponent} from './components/slots/slot-management.component';
import {SlotReserveComponent} from './components/slot-reserve/slot-reserve.component';

const routes: Routes = [
  {path: 'slots', component: SlotManagementComponent},
  {path: 'slot/:runnerId/reserve', component: SlotReserveComponent}
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class SlotRoutingModule { }
