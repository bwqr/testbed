import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { SlotRoutingModule } from './slot-routing.module';
import { SlotManagementComponent } from './components/slots/slot-management.component';
import { SlotReserveComponent } from './components/slot-reserve/slot-reserve.component';


@NgModule({
  declarations: [SlotManagementComponent, SlotReserveComponent],
  imports: [
    CommonModule,
    SlotRoutingModule
  ]
})
export class SlotModule { }
