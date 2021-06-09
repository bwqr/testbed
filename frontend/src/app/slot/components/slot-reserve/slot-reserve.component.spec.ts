import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SlotReserveComponent } from './slot-reserve.component';

describe('SlotReserveComponent', () => {
  let component: SlotReserveComponent;
  let fixture: ComponentFixture<SlotReserveComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ SlotReserveComponent ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(SlotReserveComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
