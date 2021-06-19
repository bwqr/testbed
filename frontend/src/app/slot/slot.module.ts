import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { SlotRoutingModule } from './slot-routing.module';
import { SlotsComponent } from './components/slots/slots.component';
import { SlotReserveComponent } from './components/slot-reserve/slot-reserve.component';
import {CoreModule} from '../core/core.module';
import {SharedModule} from '../shared/shared.module';
import {FormsModule} from '@angular/forms';


@NgModule({
  declarations: [SlotsComponent, SlotReserveComponent],
  imports: [
    CommonModule,
    SlotRoutingModule,
    CoreModule,
    SharedModule,
    FormsModule
  ]
})
export class SlotModule { }
