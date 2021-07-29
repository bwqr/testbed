import { ComponentFixture, TestBed } from '@angular/core/testing';

import { Controller } from './controller.component';

describe('ReceiverValuesComponent', () => {
  let component: Controller;
  let fixture: ComponentFixture<Controller>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ Controller ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(Controller);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
