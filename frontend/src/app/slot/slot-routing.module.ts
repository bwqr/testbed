import { NgModule } from '@angular/core';
import { Routes, RouterModule } from '@angular/router';
import {SlotsComponent} from './components/slots/slots.component';
import {SlotReserveComponent} from './components/slot-reserve/slot-reserve.component';

const routes: Routes = [
  {path: 'slots', component: SlotsComponent},
  {path: 'slot/:controllerId/reserve', component: SlotReserveComponent}
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class SlotRoutingModule { }
