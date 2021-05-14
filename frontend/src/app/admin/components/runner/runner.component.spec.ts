import { ComponentFixture, TestBed } from '@angular/core/testing';

import { Runner } from './runner.component';

describe('ReceiverValuesComponent', () => {
  let component: Runner;
  let fixture: ComponentFixture<Runner>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ Runner ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(Runner);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
